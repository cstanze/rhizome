use crate::AppState;

use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Json},
};
use serde_json::{Value, json};
use tracing::debug;

// PUT /temperature/nozzle
pub async fn nozzle_temperature_handler(
  state: State<AppState>,
  Json(payload): Json<Value>,
) -> impl IntoResponse {
  if let Some(target) = payload.get("target").and_then(|v| v.as_u64()) {
    if target > 300 {
      return (
        StatusCode::BAD_REQUEST,
        Json(json!({
          "error": "target temperature must be between 0 and 300"
        })),
      );
    }

    state
      .printer
      .send_gcode(&format!("M104 S{}", target as u32))
      .await
      .ok();
    debug!("set nozzle temperature to {}°C", target);

    (StatusCode::OK, Json(payload))
  } else {
    (
      StatusCode::BAD_REQUEST,
      Json(json!({
        "error": "missing or invalid 'target' field, must be an unsigned integer"
      })),
    )
  }
}

// PUT /temperature/bed
pub async fn bed_temperature_handler(
  state: State<AppState>,
  Json(payload): Json<Value>,
) -> impl IntoResponse {
  if let Some(target) = payload.get("target").and_then(|v| v.as_u64()) {
    if target > 300 {
      return (
        StatusCode::BAD_REQUEST,
        Json(json!({
          "error": "target temperature must be between 0 and 300"
        })),
      );
    }

    state
      .printer
      .send_gcode(&format!("M140 S{}", target as u32))
      .await
      .ok();
    debug!("set nozzle temperature to {}°C", target);

    (StatusCode::OK, Json(payload))
  } else {
    (
      StatusCode::BAD_REQUEST,
      Json(json!({
        "error": "missing or invalid 'target' field, must be an unsigned integer"
      })),
    )
  }
}

// PUT /fan
pub async fn fan_speed_handler(
  state: State<AppState>,
  Json(payload): Json<Value>,
) -> impl IntoResponse {
  if let Some(fan) = payload.get("fan").and_then(|v| v.as_str()) {
    if let Some(speed) = payload.get("speed").and_then(|v| v.as_u64()) {
      if speed > 255 {
        return (
          StatusCode::BAD_REQUEST,
          Json(json!({
            "error": "invalid 'speed' parameter, must be an unsigned integer [0-255]"
          })),
        );
      }

      let gcode = match fan {
        "part_cooling" => format!("M106 P1 S{}", speed),
        "auxiliary" => format!("M106 P2 S{}", speed),
        _ => {
          return (
            StatusCode::BAD_REQUEST,
            Json(json!({
              "error": "invalid 'fan' parameter, must be either 'part_cooling' or 'auxiliary'"
            })),
          );
        }
      };

      if speed == 0 {
        // Bambu firmware does not consistently treat S0 as
        // fully off across firmware versions.
        state.printer.send_gcode("M107").await.ok();
      } else {
        state.printer.send_gcode(&gcode).await.ok();
      }

      debug!("set {} fan speed to {}", fan, speed);

      (StatusCode::OK, Json(payload))
    } else {
      (
        StatusCode::BAD_REQUEST,
        Json(json!({
          "error": "missing or invalid 'speed' parameter, must be an unsigned integer [0-255]"
        })),
      )
    }
  } else {
    (
      StatusCode::BAD_REQUEST,
      Json(json!({
        "error": "missing or invalid 'fan' parameter, must be one of 'part_cooling' or 'auxiliary'"
      })),
    )
  }
}

// PUT /led
pub async fn led_handler(state: State<AppState>, Json(payload): Json<Value>) -> impl IntoResponse {
  // assume A1, only one LED which we'll default to `chamber_light`
  if let Some(light_state) = payload.get("state").and_then(|v| v.as_bool()) {
    let cmd = json!({
      "system": {
        "sequence_id": "0",
        "command": "ledctrl",
        "led_node": "chamber_light",
        "led_mode": if light_state { "on" } else { "off" },
        "led_on_time": 500,
        "led_off_time": 500,
        "loop_times": 1
      }
    });
    if state.printer.send_command(cmd).await.is_ok() {
      debug!(
        "turned {} chamber light",
        if light_state { "on" } else { "off" }
      );

      (StatusCode::OK, Json(json!({})))
    } else {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
          "error": "failed to set LED state"
        })),
      )
    }
  } else {
    (
      StatusCode::BAD_REQUEST,
      Json(json!({
        "error": "missing or invalid 'state' parameter, must be a boolean"
      })),
    )
  }
}
