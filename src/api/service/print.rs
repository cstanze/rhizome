use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintStartRequest {
  pub path: String
}

pub fn verify_print_start_request(req: &PrintStartRequest) -> bool {
  // basically, path must start with "/sdcard/"
  req.path.starts_with("/sdcard/")
}
