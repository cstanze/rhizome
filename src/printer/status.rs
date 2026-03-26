use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrinterStatus {
  pub nozzle_temp: f64,     // deg C, actual
  pub nozzle_target: f64,   // deg C, setpoint
  pub bed_temp: f64,        // deg C, actual
  pub bed_target: f64,      // deg C, setpoint
  pub gcode_state: String,  // IDLE | RUNNING | PAUSE | FAILED | FINISH
  pub subtask_name: String, // current filename
  pub layer_num: u32,
  pub total_layers: u32,
  pub progress: u32,      // %
  pub remaining_min: u32, // estimated minutes left
  pub speed_level: u8,        // 1=silent 2=standard 3=sport 4=ludicrous
  pub speed_magnitude: u32,       // speed override %
}

impl PrinterStatus {
  pub fn merge_update(&mut self, v: &Value) {
    if let Some(f) = as_f64(v, "nozzle_temper") {
      self.nozzle_temp = f;
    }
    if let Some(f) = as_f64(v, "nozzle_target_temper") {
      self.nozzle_target = f;
    }
    if let Some(f) = as_f64(v, "bed_temper") {
      self.bed_temp = f;
    }
    if let Some(f) = as_f64(v, "bed_target_temper") {
      self.bed_target = f;
    }
    if let Some(s) = as_str(v, "gcode_state") {
      self.gcode_state = s;
    }
    if let Some(s) = as_str(v, "subtask_name") {
      self.subtask_name = s;
    }
    if let Some(n) = as_u32(v, "layer_num") {
      self.layer_num = n;
    }
    if let Some(n) = as_u32(v, "total_layer_num") {
      self.total_layers = n;
    }
    if let Some(n) = as_u32(v, "mc_percent") {
      self.progress = n;
    }
    if let Some(n) = as_u32(v, "mc_remaining_time") {
      self.remaining_min = n;
    }
    if let Some(n) = as_u8(v, "spd_lvl") {
      self.speed_level = n;
    }
    if let Some(n) = as_u32(v, "spd_mag") {
      self.speed_magnitude = n;
    }
  }
}

fn as_f64(v: &Value, k: &str) -> Option<f64> {
  v.get(k)?.as_f64()
}

fn as_str(v: &Value, k: &str) -> Option<String> {
  Some(v.get(k)?.as_str()?.to_string())
}

fn as_u32(v: &Value, k: &str) -> Option<u32> {
  Some(v.get(k)?.as_u64()? as u32)
}

fn as_u8(v: &Value, k: &str) -> Option<u8> {
  Some(v.get(k)?.as_u64()? as u8)
}
