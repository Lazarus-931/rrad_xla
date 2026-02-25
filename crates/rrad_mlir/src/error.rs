use rrad_pjrt::error::PJRTError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RradMlirError {
    #[error("invalid module: {0}")]
    InvalidModule(String),

    #[error("rrad_pjrt_runtime error: {0}")]
    Pjrt(String),
}

impl<'rt> From<PJRTError<'rt>> for RradMlirError {
    fn from(value: PJRTError<'rt>) -> Self {
        let msg = value.message().unwrap_or_else(|_| "unknown rrad_pjrt_runtime error".to_string());
        Self::Pjrt(msg)
    }
}
