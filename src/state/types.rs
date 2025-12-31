use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State tracking for incremental updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// State file format version
    pub version: String,

    /// Last time state was updated
    pub last_updated: DateTime<Utc>,

    /// Per-source state tracking
    pub sources: HashMap<String, SourceState>,
}

/// State for a specific source (Git repo, TODO file, or notes directory)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SourceState {
    #[serde(rename = "git")]
    Git {
        last_checked: DateTime<Utc>,
        default_branch: String,
        branches: HashMap<String, BranchState>,
    },
    #[serde(rename = "todo")]
    Todo {
        last_checked: DateTime<Utc>,
        last_modified: DateTime<Utc>,
        item_hashes: Vec<String>,
    },
    #[serde(rename = "notes")]
    Notes {
        last_checked: DateTime<Utc>,
        files: HashMap<String, DateTime<Utc>>,
    },
}

/// State for a Git branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchState {
    /// Last commit hash seen on this branch
    pub last_commit: String,

    /// Last time this branch was checked
    pub last_seen: DateTime<Utc>,

    /// First time this branch was seen (for new branch detection)
    pub first_seen: Option<DateTime<Utc>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            last_updated: Utc::now(),
            sources: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_default() {
        let state = State::default();
        assert_eq!(state.version, "1.0");
        assert_eq!(state.sources.len(), 0);
    }

    #[test]
    fn test_state_serialization() {
        let state = State::default();
        let json = serde_json::to_string(&state).unwrap();
        let parsed: State = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "1.0");
    }

    #[test]
    fn test_source_state_git_serialization() {
        let mut branches = HashMap::new();
        branches.insert(
            "main".to_string(),
            BranchState {
                last_commit: "abc123".to_string(),
                last_seen: Utc::now(),
                first_seen: None,
            },
        );

        let git_state = SourceState::Git {
            last_checked: Utc::now(),
            default_branch: "main".to_string(),
            branches,
        };

        let json = serde_json::to_string(&git_state).unwrap();
        let parsed: SourceState = serde_json::from_str(&json).unwrap();

        match parsed {
            SourceState::Git {
                default_branch, ..
            } => {
                assert_eq!(default_branch, "main");
            }
            _ => panic!("Expected Git variant"),
        }
    }

    #[test]
    fn test_source_state_todo_serialization() {
        let todo_state = SourceState::Todo {
            last_checked: Utc::now(),
            last_modified: Utc::now(),
            item_hashes: vec!["hash1".to_string(), "hash2".to_string()],
        };

        let json = serde_json::to_string(&todo_state).unwrap();
        let parsed: SourceState = serde_json::from_str(&json).unwrap();

        match parsed {
            SourceState::Todo { item_hashes, .. } => {
                assert_eq!(item_hashes.len(), 2);
            }
            _ => panic!("Expected Todo variant"),
        }
    }

    #[test]
    fn test_source_state_notes_serialization() {
        let mut files = HashMap::new();
        files.insert("note1.md".to_string(), Utc::now());

        let notes_state = SourceState::Notes {
            last_checked: Utc::now(),
            files,
        };

        let json = serde_json::to_string(&notes_state).unwrap();
        let parsed: SourceState = serde_json::from_str(&json).unwrap();

        match parsed {
            SourceState::Notes { files, .. } => {
                assert_eq!(files.len(), 1);
            }
            _ => panic!("Expected Notes variant"),
        }
    }
}
