use crate::{AppState, printer::PrinterSpeed};

use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintStartRequest {
  pub path: String,
}

pub fn verify_print_start_request(req: &PrintStartRequest) -> bool {
  // basically, path must start with "/sdcard/"
  req.path.starts_with("/sdcard/")
}

// GET /print
pub async fn print_status_handler(state: State<AppState>) -> Json<Value> {
  let status = state.printer.status().await;

  if let Some(status) = status {
    Json(json!({
      "print": {
        "state": status.gcode_state,
        "filename": status.subtask_name,
        "current_layer": status.layer_num,
        "total_layers": status.total_layers,
        "progress": status.progress,
        "remaining_min": status.remaining_min,
        "speed_level": status.speed_level,
        "speed_magnitude": status.speed_magnitude,
      }
    }))
  } else {
    Json(json!({
      "print": null
    }))
  }
}

// POST /print
pub async fn start_print_handler(
  state: State<AppState>,
  Json(payload): Json<PrintStartRequest>,
) -> impl IntoResponse {
  if !verify_print_start_request(&payload) {
    return (
      StatusCode::BAD_REQUEST,
      Json(json!({
        "error": "path must begin with /sdcard/"
      })),
    );
  }

  if state.printer.start_print(&payload.path).await.is_ok() {
    info!("started print job: {}", payload.path);
  } else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "error": "failed to start print job"
      })),
    );
  }

  (StatusCode::OK, Json(serde_json::to_value(payload).unwrap()))
}

// PUT /print/pause
pub async fn pause_print_handler(state: State<AppState>) -> impl IntoResponse {
  if state.printer.pause_print().await.is_ok() {
    info!("paused print job");
  } else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "error": "failed to pause print job"
      })),
    );
  }

  (StatusCode::OK, Json(json!({})))
}

// PUT /print/resume
pub async fn resume_print_handler(state: State<AppState>) -> impl IntoResponse {
  if state.printer.resume_print().await.is_ok() {
    info!("resumed print job");
  } else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "error": "failed to resume print job"
      })),
    );
  }

  (StatusCode::OK, Json(json!({})))
}

// PUT /print/stop
pub async fn stop_print_handler(state: State<AppState>) -> impl IntoResponse {
  if state.printer.stop_print().await.is_ok() {
    info!("stopped print job");
  } else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "error": "failed to stop print job"
      })),
    );
  }

  (StatusCode::OK, Json(json!({})))
}

// PUT /print/speed
pub async fn set_print_speed_handler(
  state: State<AppState>,
  Json(payload): Json<Value>,
) -> impl IntoResponse {
  if let Some(speed) = payload.get("speed").and_then(|s| s.as_u64()) {
    if !(1..=4).contains(&speed) {
      return (
        StatusCode::BAD_REQUEST,
        Json(json!({
          "error": "invalid 'speed' parameter, must be an unsigned integer [1-4]"
        })),
      );
    }

    if state
      .printer
      .set_speed(PrinterSpeed::from(speed as u8))
      .await
      .is_ok()
    {
      info!("set print speed to level {}", speed);
    } else {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
          "error": "failed to set print speed"
        })),
      );
    }

    (StatusCode::OK, Json(json!({})))
  } else {
    (
      StatusCode::BAD_REQUEST,
      Json(json!({
        "error": "missing or invalid 'speed' parameter, must be an unsigned integer [1-4]"
      })),
    )
  }
}
