use crate::AppState;

use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Json},
};
use serde_json::{Value, json};
use tracing::info;

// POST /gcode
pub async fn send_gcode_handler(
  state: State<AppState>,
  Json(payload): Json<Value>,
) -> impl IntoResponse {
  if let Some(command) = payload.get("lines").and_then(|v| v.as_str()) {
    state.printer.send_gcode(command).await.ok();
    info!("sent G-code command: {}", command);

    (StatusCode::OK, Json(payload))
  } else {
    (
      StatusCode::BAD_REQUEST,
      Json(json!({
        "error": "missing or invalid 'lines' field, must be a string"
      })),
    )
  }
}
