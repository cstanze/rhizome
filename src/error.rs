use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("connection to printer timed out (time taken > 5000ms)")]
  ConnectionTimeout,

  #[error(transparent)]
  SystemTime(#[from] std::time::SystemTimeError),

  #[error(transparent)]
  Anyhow(#[from] anyhow::Error),

  #[error(transparent)]
  Io(#[from] std::io::Error),
}
