use crate::api::AppState;
use axum::{
  body::Body,
  extract::{Request, State},
  http::StatusCode,
  middleware::Next,
  response::Response,
};
use serde_json::json;
use tracing::warn;

pub async fn printer_gateway(
  State(state): State<AppState>,
  request: Request,
  next: Next,
) -> Response {
  if !state.printer.is_connected().await {
    warn!("printer is not connected, rejecting request");
    return Response::builder()
      .status(StatusCode::SERVICE_UNAVAILABLE)
      .body(Body::from(json!({}).to_string()))
      .unwrap();
  }

  next.run(request).await
}
