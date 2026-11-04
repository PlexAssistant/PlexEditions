use std::time::Duration;

/// Failures that stop a run, not a single movie.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("plex {status} {url}")]
    Plex {
        status: reqwest::StatusCode,
        url: String,
    },
    #[error("plex response: {0}")]
    Decode(String),
    #[error("{0}")]
    Config(String),
    #[error("websocket: {0}")]
    WebSocket(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// How long any single HTTP call may take before it is abandoned.
pub const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
