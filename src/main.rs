mod api;
mod error;
mod printer;

use crate::api::AppState;
use crate::error::AppError;
use crate::printer::{PrinterClient, PrinterConfig, PrinterStatus};

use axum::Router;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::{task, time};
use tracing::{debug, error, info, level_filters::LevelFilter, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const IP: &str = "10.0.1.110";
const SERIAL: &'static str = "03900D5B1806547";
const CODE: &'static str = "25eb0912";

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
  let client = Arc::new(RwLock::new(client));

  let (broadcast_tx, mut broadcast_rx) = broadcast::channel::<Option<PrinterStatus>>(64);

  let broadcast_task_client = client.clone();
  let broacast_task_tx = broadcast_tx.clone();
  task::spawn(async move {
    loop {
      let status = broadcast_task_client.read().await.status().await;
      if let Err(e) = broacast_task_tx.send(status) {
        error!("failed to broadcast printer status: {e}");
      }
      time::sleep(Duration::from_secs(2)).await;
    }
  });

  task::spawn(async move {
    while let Ok(status) = broadcast_rx.recv().await {
      debug!("broadcasting status: {status:?}");
    }
  });

  info!("starting API service...");
  let v1_api = Router::new()
    .route(
      "/status",
      axum::routing::get(api::v1::status::status_handler),
    )
    .route(
      "/print",
      axum::routing::get(api::v1::print::print_status_handler),
    )
    .route(
      "/print",
      axum::routing::post(api::v1::print::start_print_handler),
    )
    .route(
      "/print/pause",
      axum::routing::put(api::v1::print::pause_print_handler),
    )
    .route(
      "/print/resume",
      axum::routing::put(api::v1::print::resume_print_handler),
    )
    .route(
      "/print/stop",
      axum::routing::put(api::v1::print::stop_print_handler),
    );
  let app = Router::new().nest("/v1", v1_api).with_state(AppState {
    printer: client,
    broadcast_tx,
  });

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
  axum::serve(listener, app).await?;

  Ok(())
}
