use thiserror::Error;

/// Chronicle error types
#[derive(Error, Debug)]
pub enum ChronicleError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("State file error: {0}")]
    State(String),

    #[error("Collector error: {0}")]
    Collector(String),

    #[error("Renderer error: {0}")]
    Renderer(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type for Chronicle operations
pub type Result<T> = std::result::Result<T, ChronicleError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_config() {
        let err = ChronicleError::Config("test error".to_string());
        assert_eq!(err.to_string(), "Configuration error: test error");
    }

    #[test]
    fn test_error_display_state() {
        let err = ChronicleError::State("test state error".to_string());
        assert_eq!(err.to_string(), "State file error: test state error");
    }

    #[test]
    fn test_error_display_collector() {
        let err = ChronicleError::Collector("test collector error".to_string());
        assert_eq!(err.to_string(), "Collector error: test collector error");
    }

    #[test]
    fn test_error_display_renderer() {
        let err = ChronicleError::Renderer("test renderer error".to_string());
        assert_eq!(err.to_string(), "Renderer error: test renderer error");
    }
}
