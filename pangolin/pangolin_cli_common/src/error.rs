use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("API Request Failed: {0}")]
    ApiError(String),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Configuration Error: {0}")]
    ConfigError(String),
    #[error("Authentication Failed: {0}")]
    AuthError(String),
    #[error("Internal Error: {0}")]
    Internal(String),
}
