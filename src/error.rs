use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error(transparent)]
  SystemTime(#[from] std::time::SystemTimeError),

  #[error(transparent)]
  Anyhow(#[from] anyhow::Error),

  #[error(transparent)]
  Io(#[from] std::io::Error),
}
