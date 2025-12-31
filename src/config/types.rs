use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Chronicle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directory where chronicle files are written
    pub output_dir: PathBuf,

    /// Path to state file for tracking last runs
    pub state_file: PathBuf,

    /// Git repositories to track
    pub repos: Vec<PathBuf>,

    /// TODO/Inbox files to parse
    pub todo_files: Vec<PathBuf>,

    /// Directories containing note files
    pub notes_dirs: Vec<PathBuf>,

    /// Collection limits
    pub limits: Limits,

    /// Display settings
    pub display: Display,
}

/// Limits for data collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    /// Maximum commits to collect per repository
    pub max_commits: usize,

    /// Maximum changed files to show
    pub max_changed_files: usize,

    /// Maximum note files to include
    pub max_note_files: usize,

    /// Maximum characters per item (TODOs, notes)
    pub max_chars_per_item: usize,
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Display {
    /// Show author names on commits (useful for teams, disable for solo)
    pub show_authors: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./chronicles"),
            state_file: PathBuf::from("./.chronicle-state.json"),
            repos: vec![PathBuf::from(".")],
            todo_files: Vec::new(),
            notes_dirs: Vec::new(),
            limits: Limits::default(),
            display: Display::default(),
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_commits: 50,
            max_changed_files: 80,
            max_note_files: 30,
            max_chars_per_item: 2000,
        }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self { show_authors: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.output_dir, PathBuf::from("./chronicles"));
        assert_eq!(config.repos, vec![PathBuf::from(".")]);
        assert_eq!(config.limits.max_commits, 50);
        assert_eq!(config.limits.max_changed_files, 80);
        assert_eq!(config.limits.max_note_files, 30);
        assert_eq!(config.limits.max_chars_per_item, 2000);
        assert_eq!(config.display.show_authors, true);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.limits.max_commits, 50);
        assert_eq!(parsed.display.show_authors, true);
    }

    #[test]
    fn test_limits_default() {
        let limits = Limits::default();
        assert_eq!(limits.max_commits, 50);
        assert_eq!(limits.max_changed_files, 80);
        assert_eq!(limits.max_note_files, 30);
        assert_eq!(limits.max_chars_per_item, 2000);
    }

    #[test]
    fn test_display_default() {
        let display = Display::default();
        assert_eq!(display.show_authors, true);
    }
}
