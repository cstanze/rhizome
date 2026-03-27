mod api;
mod error;
mod printer;

use crate::api::AppState;
use crate::error::AppError;
use crate::printer::{PrinterClient, PrinterConfig};
use tokio::sync::mpsc;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const IP: &str = "10.0.1.110";
const SERIAL: &str = "03900D5B1806547";
const CODE: &str = "25eb0912";

#[tokio::main]
async fn main() -> anyhow::Result<(), AppError> {
  tracing_subscriber::registry()
    .with(fmt::layer())
    .with(
      EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .unwrap(),
    )
    .init();

  // Required once at startup when using rustls 0.23
  rustls::crypto::ring::default_provider()
    .install_default()
    .expect("failed to install rustls crypto provider");

  let (event_tx, event_rx) = mpsc::channel(64);
  let printer_config = PrinterConfig {
    ip: IP.into(),
    serial: SERIAL.into(),
    access_code: CODE.into(),
  };

  info!("connecting to printer...");
  let client = PrinterClient::connect(printer_config, event_tx).await?;

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
  axum::serve(listener, api::build(client, event_rx)).await?;

  Ok(())
}
