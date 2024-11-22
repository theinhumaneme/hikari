use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing or empty field: {0}")]
    MissingField(String),

    #[error("Failed to read configuration file: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
}
