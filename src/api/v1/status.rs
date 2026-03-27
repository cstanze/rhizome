use crate::AppState;

use axum::{extract::State, response::Json};
use serde_json::{Value, json};

pub async fn status_handler(state: State<AppState>) -> Json<Value> {
  let status = state.printer.status().await;

  Json(json!({
    "status": status
  }))
}
