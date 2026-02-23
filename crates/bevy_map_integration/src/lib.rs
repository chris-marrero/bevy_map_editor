pub mod manager;
pub mod plugin_meta;
pub mod prelude;
pub mod registry;

#[cfg(feature = "editor")]
pub mod editor;

use std::fmt;

/// Errors that can occur during plugin integration operations.
#[derive(Debug)]
pub enum IntegrationError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl fmt::Display for IntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntegrationError::IoError(msg) => write!(f, "IO error: {msg}"),
            IntegrationError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            IntegrationError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
        }
    }
}

impl std::error::Error for IntegrationError {}

impl From<std::io::Error> for IntegrationError {
    fn from(err: std::io::Error) -> Self {
        IntegrationError::IoError(err.to_string())
    }
}

impl From<toml::de::Error> for IntegrationError {
    fn from(err: toml::de::Error) -> Self {
        IntegrationError::ParseError(err.to_string())
    }
}
