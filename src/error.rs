use thiserror::Error;

#[derive(Debug, Error)]
pub enum HostError {
    #[error("invalid configuration: {0}")]
    Configuration(String),
    #[error("request boundary rejected")]
    Boundary,
    #[error("session authorization failed")]
    Unauthorized,
    #[error("request budget exhausted")]
    Budget,
    #[error("request replay rejected")]
    Replay,
    #[error("requested view is not available")]
    Unavailable,
    #[error("invalid request: {0}")]
    Request(String),
    #[error("I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON failed: {0}")]
    Json(#[from] serde_json::Error),
}
