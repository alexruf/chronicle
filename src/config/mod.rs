//! Configuration module
//!
//! Handles loading and saving of chronicle.toml configuration files.
//! Defines Config, Limits, and Display types.

mod types;

#[allow(unused_imports)]
pub use types::{Config, Display, Limits};

use crate::error::{ChronicleError, Result};
use std::fs;
use std::path::Path;

/// Load configuration from a TOML file
pub fn load(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path).map_err(|e| {
        ChronicleError::Config(format!(
            "Cannot read config from '{}': {}. Run 'chronicle config init' to create one.",
            path.display(),
            e
        ))
    })?;

    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

/// Save configuration to a TOML file
pub fn save(config: &Config, path: &Path) -> Result<()> {
    let toml = toml::to_string_pretty(config)
        .map_err(|e| ChronicleError::Config(format!("Failed to serialize config: {}", e)))?;

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, toml)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_valid_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("chronicle.toml");

        let config = Config::default();
        save(&config, &config_path).unwrap();

        let loaded = load(&config_path).unwrap();
        assert_eq!(loaded.limits.max_commits, 50);
        assert_eq!(loaded.display.show_authors, true);
    }

    #[test]
    fn test_load_missing_config() {
        let result = load(Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Run 'chronicle config init'"));
    }

    #[test]
    fn test_save_creates_directories() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("nested/dir/chronicle.toml");

        let config = Config::default();
        save(&config, &config_path).unwrap();

        assert!(config_path.exists());
    }

    #[test]
    fn test_roundtrip() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("chronicle.toml");

        let mut config = Config::default();
        config.repos.push("/test/repo".into());
        config.todo_files.push("/test/todo.md".into());

        save(&config, &config_path).unwrap();
        let loaded = load(&config_path).unwrap();

        assert_eq!(loaded.repos.len(), 1);
        assert_eq!(loaded.todo_files.len(), 1);
    }
}
