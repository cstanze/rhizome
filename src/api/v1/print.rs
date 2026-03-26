use crate::AppState;

use super::super::service::print::{PrintStartRequest, verify_print_start_request};
use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Json},
};
use serde_json::{Value, json};
use tracing::info;

pub async fn print_status_handler(state: State<AppState>) -> Json<Value> {
  let status = state.printer.read().await.status().await;

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

  if let Ok(_) = state.printer.write().await.start_print(&payload.path).await {
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

pub async fn pause_print_handler(state: State<AppState>) -> impl IntoResponse {
  if let Ok(_) = state.printer.write().await.send_command(json!({
    "print": {
      "command": "pause_print"
    }
  })).await {
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

pub async fn resume_print_handler(state: State<AppState>) -> impl IntoResponse {
  if let Ok(_) = state.printer.write().await.send_command(json!({
    "print": {
      "command": "resume_print"
    }
  })).await {
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

pub async fn stop_print_handler(state: State<AppState>) -> impl IntoResponse {
  if let Ok(_) = state.printer.write().await.send_command(json!({
    "print": {
      "command": "stop_print"
    }
  })).await {
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
