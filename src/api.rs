pub mod middleware;
pub mod v1;

use axum::routing::{get, post, put};
use tokio::{
  sync::{broadcast, mpsc},
  task,
};
use tracing::{debug, error};

use crate::api::middleware::printer_gateway;
use crate::printer::PrinterEvent;
pub use crate::printer::{PrinterClient, PrinterStatus};

#[derive(Clone)]
pub struct AppState {
  pub printer: PrinterClient,
  pub broadcast_tx: broadcast::Sender<Option<PrinterStatus>>,
}

pub fn build(client: PrinterClient, mut event_rx: mpsc::Receiver<PrinterEvent>) -> axum::Router {
  let (broadcast_tx, keepalive_rx) = broadcast::channel::<Option<PrinterStatus>>(64);

  let tx = broadcast_tx.clone();
  task::spawn(async move {
    let _keepalive = keepalive_rx; // keep the receiver alive to prevent the channel from closing
    
    while let Some(event) = event_rx.recv().await {
      debug!("received printer event: {event:?}");

      let status = match event {
        PrinterEvent::StatusUpdate(s) => Some(s),
        PrinterEvent::Disconnected => None,
      };

      if let Err(e) = tx.send(status) {
        error!("failed to broadcast printer status update: {e}");
      }
    }
  });

  let state = AppState {
    printer: client.clone(),
    broadcast_tx,
  };
  axum::Router::new()
    .nest("/v1", v1_router())
    .layer(axum::middleware::from_fn_with_state(
      state.clone(),
      printer_gateway,
    ))
    .with_state(state)
}

fn v1_router() -> axum::Router<AppState> {
  axum::Router::new()
    .route("/status", get(v1::status::status_handler))
    .route("/print", get(v1::print::print_status_handler))
    .route("/print", post(v1::print::start_print_handler))
    .route("/print/pause", put(v1::print::pause_print_handler))
    .route("/print/resume", put(v1::print::resume_print_handler))
    .route("/print/stop", put(v1::print::stop_print_handler))
    .route("/print/speed", put(v1::print::set_print_speed_handler))
    .route(
      "/temperature/nozzle",
      put(v1::peripherals::nozzle_temperature_handler),
    )
    .route(
      "/temperature/bed",
      put(v1::peripherals::bed_temperature_handler),
    )
    .route("/fan", put(v1::peripherals::fan_speed_handler))
    .route("/led", put(v1::peripherals::led_handler))
    .route("/gcode", post(v1::gcode::send_gcode_handler))
    .route("/ws", get(v1::ws::handler))
}
