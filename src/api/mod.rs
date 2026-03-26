pub mod service;
pub mod v1;

use tokio::sync::broadcast;
use tokio::sync::RwLock;
use std::sync::Arc;

pub use crate::printer::{PrinterClient, PrinterStatus};

#[derive(Clone)]
pub struct AppState {
  pub printer: Arc<RwLock<PrinterClient>>,
  pub broadcast_tx: broadcast::Sender<Option<PrinterStatus>>,
}
