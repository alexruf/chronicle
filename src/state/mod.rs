//! State persistence module
//!
//! Tracks "last run" timestamps per source to enable incremental updates.
//! Stores state in JSON format (.chronicle-state.json).

pub mod types;

pub use types::{BranchState, SourceState, State};

use crate::error::{ChronicleError, Result};
use chrono::Utc;
use std::fs;
use std::path::Path;

/// Load state from JSON file, returning default state if file doesn't exist
pub fn load(path: &Path) -> Result<State> {
    if !path.exists() {
        return Ok(State::default());
    }

    let content = fs::read_to_string(path).map_err(|e| {
        ChronicleError::State(format!(
            "Cannot read state from '{}': {}",
            path.display(),
            e
        ))
    })?;

    let state: State = serde_json::from_str(&content)?;
    Ok(state)
}

/// Save state to JSON file with pretty formatting
pub fn save(state: &State, path: &Path) -> Result<()> {
    // Update last_updated timestamp
    let mut updated_state = state.clone();
    updated_state.last_updated = Utc::now();

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let json = serde_json::to_string_pretty(&updated_state)?;
    fs::write(path, json).map_err(|e| {
        ChronicleError::State(format!("Cannot write state to '{}': {}", path.display(), e))
    })?;

    Ok(())
}

/// Get state for a specific source by name
pub fn get_source<'a>(state: &'a State, source_name: &str) -> Option<&'a SourceState> {
    state.sources.get(source_name)
}

/// Update state for a specific source
pub fn update_source(state: &mut State, source_name: String, source_state: SourceState) {
    state.sources.insert(source_name, source_state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_load_nonexistent_returns_default() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("nonexistent.json");

        let state = load(&state_path).unwrap();
        assert_eq!(state.version, "1.0");
        assert_eq!(state.sources.len(), 0);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        let mut state = State::default();
        let git_state = SourceState::Git {
            last_checked: Utc::now(),
            default_branch: "main".to_string(),
            branches: HashMap::new(),
        };
        update_source(&mut state, "test-repo".to_string(), git_state);

        save(&state, &state_path).unwrap();
        assert!(state_path.exists());

        let loaded = load(&state_path).unwrap();
        assert_eq!(loaded.version, "1.0");
        assert_eq!(loaded.sources.len(), 1);
        assert!(loaded.sources.contains_key("test-repo"));
    }

    #[test]
    fn test_save_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("state.json");

        let state = State::default();
        save(&state, &state_path).unwrap();
        assert!(state_path.exists());
    }

    #[test]
    fn test_save_updates_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        let original_state = State::default();
        let original_time = original_state.last_updated;

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        save(&original_state, &state_path).unwrap();
        let loaded = load(&state_path).unwrap();

        assert!(loaded.last_updated > original_time);
    }

    #[test]
    fn test_get_source() {
        let mut state = State::default();
        let git_state = SourceState::Git {
            last_checked: Utc::now(),
            default_branch: "main".to_string(),
            branches: HashMap::new(),
        };
        update_source(&mut state, "test-repo".to_string(), git_state);

        let retrieved = get_source(&state, "test-repo");
        assert!(retrieved.is_some());

        let missing = get_source(&state, "missing-repo");
        assert!(missing.is_none());
    }

    #[test]
    fn test_update_source() {
        let mut state = State::default();
        assert_eq!(state.sources.len(), 0);

        let todo_state = SourceState::Todo {
            last_checked: Utc::now(),
            last_modified: Utc::now(),
            item_hashes: vec!["hash1".to_string()],
        };
        update_source(&mut state, "todo.txt".to_string(), todo_state);

        assert_eq!(state.sources.len(), 1);
        assert!(state.sources.contains_key("todo.txt"));
    }

    #[test]
    fn test_load_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("invalid.json");

        fs::write(&state_path, "not valid json").unwrap();

        let result = load(&state_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ChronicleError::Json(_)));
    }
}
