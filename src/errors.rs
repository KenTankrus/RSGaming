use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to (de)serialize JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Grand Exchange API error: {0}")]
    GeApi(String),

    #[error("Telegram error: {0}")]
    Telegram(String),

    #[error("Could not determine the application data directory")]
    NoDataDir,

    #[error("{0}")]
    Other(String),
}

pub type AppResult<T> = Result<T, AppError>;
