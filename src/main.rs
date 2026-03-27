mod api;
mod config;
mod error;
mod printer;

use crate::api::AppState;
use crate::config::Config;
use crate::error::AppError;
use crate::printer::PrinterClient;
use tokio::sync::mpsc;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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

  // required once at startup when using rustls 0.23
  rustls::crypto::ring::default_provider()
    .install_default()
    .expect("failed to install rustls crypto provider");

  let (event_tx, event_rx) = mpsc::channel(64);
  let config = Config::load()?;
  let server_addr = config.server_addr();
  let printer_config = config.printer;

  info!("connecting to printer...");
  let client = PrinterClient::connect(printer_config, event_tx).await?;

  let listener = tokio::net::TcpListener::bind(server_addr).await?;
  axum::serve(listener, api::build(client, event_rx)).await?;

  Ok(())
}
