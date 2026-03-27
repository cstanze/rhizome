use crate::api::AppState;
use crate::printer::PrinterStatus;
use axum::{
  extract::{
    State,
    ws::{Message, WebSocket, WebSocketUpgrade},
  },
  response::IntoResponse,
};
use std::time::Duration;
use tokio::time::{interval, timeout};

pub async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
  ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
  // subscribe before fetching current status to avoid a race where a
  // status update arrives between the fetch and the subscribe
  let mut rx = state.broadcast_tx.subscribe();

  // send current snapshot immediately so the client isn't waiting in silence
  let current = state.printer.status().await;
  if send_status(&mut socket, current).await.is_err() {
    return; // client already gone
  }

  let mut ping_interval = interval(Duration::from_secs(15));
  ping_interval.tick().await; // consume the immediate first tick

  loop {
    tokio::select! {
      // status update
      result = rx.recv() => {
        match result {
          Ok(status) => {
            if send_status(&mut socket, status).await.is_err() {
              return; // client disconnected
            }
          }
          Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
            // Client fell behind — missed n messages, but the next
            // recv() will return the oldest surviving message, which
            // is a full snapshot, so we just continue
            tracing::warn!("ws client lagged by {n} messages");
          }
          Err(tokio::sync::broadcast::error::RecvError::Closed) => {
            return; // broadcast channel shut down
          }
        }
      }

      // heartbeat
      _ = ping_interval.tick() => {
        if socket.send(Message::Ping(vec![].into())).await.is_err() {
          return; // client disconnected
        }

        // wait up to 5 seconds for the pong
        match timeout(Duration::from_secs(5), wait_for_pong(&mut socket)).await {
          Ok(true)  => {} // pong received, continue
          Ok(false) => return, // client sent something unexpected or closed
          Err(_)    => return, // timed out — drop the client
        }
      }

      // inbound messages from client (receive-only, so just drain cleanly)
      msg = socket.recv() => {
        match msg {
          Some(Ok(Message::Close(_))) | None => return,
          Some(Ok(Message::Pong(_)))         => {} // response to our ping
          Some(Ok(_))                        => {} // ignore anything else
          Some(Err(_))                       => return,
        }
      }
    }
  }
}

/// Serialize a status snapshot and send it as a JSON text frame.
async fn send_status(socket: &mut WebSocket, status: Option<PrinterStatus>) -> Result<(), ()> {
  let msg = serde_json::json!({
    "type": "status",
    "data": status
  });

  socket
    .send(Message::Text(msg.to_string().into()))
    .await
    .map_err(|_| ())
}

/// Drain the socket until we get a Pong or the connection closes.
async fn wait_for_pong(socket: &mut WebSocket) -> bool {
  loop {
    match socket.recv().await {
      Some(Ok(Message::Pong(_))) => return true,
      Some(Ok(Message::Close(_))) | None => return false,
      Some(Ok(_)) => {} // could be a stale ping response etc, keep waiting
      Some(Err(_)) => return false,
    }
  }
}
