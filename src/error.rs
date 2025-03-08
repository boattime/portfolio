use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Environment variable parsing error: {0}")]
    EnvyError(#[from] envy::Error),

    #[error("Template error: {0}")]
    TemplateError(#[from] tera::Error),

    #[error("Scheduler error: {0}")]
    SchedulerError(String),

    #[error("Generation error: {0}")]
    GenerationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = Error::Io(io_err);
        assert!(err.to_string().contains("I/O error"));

        let config_err = Error::ConfigError("invalid configuration".to_string());
        assert_eq!(
            config_err.to_string(),
            "Configuration error: invalid configuration"
        );
    }
}
