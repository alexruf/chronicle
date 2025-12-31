use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::source::{ChangeKind, Note, Repository, Todo};

/// Aggregate chronicle for a specific date/time range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chronicle {
    /// Date this chronicle represents
    pub date: NaiveDate,
    /// Start time for incremental updates (only items since this time)
    pub since: DateTime<Utc>,
    /// When this chronicle was generated
    pub generated_at: DateTime<Utc>,
    /// Git repositories with their activity
    pub repositories: Vec<Repository>,
    /// TODO items
    pub todos: Vec<Todo>,
    /// Note updates
    pub notes: Vec<Note>,
}

/// Summary statistics for a chronicle
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChronicleStats {
    /// Number of repositories with activity
    pub repo_count: usize,
    /// Total number of commits
    pub commit_count: usize,
    /// Number of new branches
    pub new_branch_count: usize,
    /// Number of new TODOs
    pub todos_new: usize,
    /// Number of completed TODOs
    pub todos_completed: usize,
    /// Number of note updates
    pub notes_count: usize,
}

impl Chronicle {
    /// Compute summary statistics from the chronicle data
    pub fn stats(&self) -> ChronicleStats {
        let repo_count = self.repositories.len();
        let commit_count = self.repositories.iter().map(|r| r.commit_count()).sum();
        let new_branch_count = self.repositories.iter().map(|r| r.new_branch_count()).sum();

        let todos_new = self
            .todos
            .iter()
            .filter(|t| t.change == ChangeKind::New)
            .count();

        let todos_completed = self.todos.iter().filter(|t| t.was_completed()).count();

        let notes_count = self.notes.len();

        ChronicleStats {
            repo_count,
            commit_count,
            new_branch_count,
            todos_new,
            todos_completed,
            notes_count,
        }
    }

    /// Check if there's any activity in this chronicle
    pub fn has_activity(&self) -> bool {
        !self.repositories.is_empty() || !self.todos.is_empty() || !self.notes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::models::source::{Branch, Commit, TodoStatus};

    #[test]
    fn test_chronicle_stats_empty() {
        let chronicle = Chronicle {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            since: Utc::now(),
            generated_at: Utc::now(),
            repositories: vec![],
            todos: vec![],
            notes: vec![],
        };

        let stats = chronicle.stats();
        assert_eq!(stats.repo_count, 0);
        assert_eq!(stats.commit_count, 0);
        assert_eq!(stats.new_branch_count, 0);
        assert_eq!(stats.todos_new, 0);
        assert_eq!(stats.todos_completed, 0);
        assert_eq!(stats.notes_count, 0);
    }

    #[test]
    fn test_chronicle_stats_with_data() {
        let chronicle = Chronicle {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            since: Utc::now(),
            generated_at: Utc::now(),
            repositories: vec![
                Repository {
                    path: PathBuf::from("/test/repo1"),
                    name: "repo1".to_string(),
                    default_branch: "main".to_string(),
                    branches: vec![
                        Branch {
                            name: "main".to_string(),
                            change: ChangeKind::Modified,
                            ahead: 0,
                            behind: 0,
                            commits: vec![
                                Commit {
                                    hash: "abc1234".to_string(),
                                    message: "Commit 1".to_string(),
                                    author: "Author".to_string(),
                                    timestamp: Utc::now(),
                                    files: vec![],
                                },
                                Commit {
                                    hash: "def5678".to_string(),
                                    message: "Commit 2".to_string(),
                                    author: "Author".to_string(),
                                    timestamp: Utc::now(),
                                    files: vec![],
                                },
                            ],
                        },
                        Branch {
                            name: "feature".to_string(),
                            change: ChangeKind::New,
                            ahead: 1,
                            behind: 0,
                            commits: vec![Commit {
                                hash: "ghi9012".to_string(),
                                message: "Feature".to_string(),
                                author: "Author".to_string(),
                                timestamp: Utc::now(),
                                files: vec![],
                            }],
                        },
                    ],
                },
                Repository {
                    path: PathBuf::from("/test/repo2"),
                    name: "repo2".to_string(),
                    default_branch: "main".to_string(),
                    branches: vec![Branch {
                        name: "main".to_string(),
                        change: ChangeKind::Modified,
                        ahead: 0,
                        behind: 0,
                        commits: vec![Commit {
                            hash: "jkl3456".to_string(),
                            message: "Another commit".to_string(),
                            author: "Author".to_string(),
                            timestamp: Utc::now(),
                            files: vec![],
                        }],
                    }],
                },
            ],
            todos: vec![
                Todo {
                    content: "New task".to_string(),
                    status: TodoStatus::Pending,
                    change: ChangeKind::New,
                    previous_status: None,
                    file: PathBuf::from("todo.txt"),
                    line: 1,
                },
                Todo {
                    content: "Completed task".to_string(),
                    status: TodoStatus::Done,
                    change: ChangeKind::Modified,
                    previous_status: Some(TodoStatus::Pending),
                    file: PathBuf::from("todo.txt"),
                    line: 2,
                },
                Todo {
                    content: "Existing task".to_string(),
                    status: TodoStatus::Pending,
                    change: ChangeKind::Unchanged,
                    previous_status: Some(TodoStatus::Pending),
                    file: PathBuf::from("todo.txt"),
                    line: 3,
                },
            ],
            notes: vec![
                Note {
                    path: PathBuf::from("note1.md"),
                    change: ChangeKind::New,
                    modified_at: Utc::now(),
                    excerpt: "New note".to_string(),
                },
                Note {
                    path: PathBuf::from("note2.md"),
                    change: ChangeKind::Modified,
                    modified_at: Utc::now(),
                    excerpt: "Modified note".to_string(),
                },
            ],
        };

        let stats = chronicle.stats();
        assert_eq!(stats.repo_count, 2);
        assert_eq!(stats.commit_count, 4);
        assert_eq!(stats.new_branch_count, 1);
        assert_eq!(stats.todos_new, 1);
        assert_eq!(stats.todos_completed, 1);
        assert_eq!(stats.notes_count, 2);
    }

    #[test]
    fn test_chronicle_has_activity() {
        let empty_chronicle = Chronicle {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            since: Utc::now(),
            generated_at: Utc::now(),
            repositories: vec![],
            todos: vec![],
            notes: vec![],
        };
        assert!(!empty_chronicle.has_activity());

        let chronicle_with_repos = Chronicle {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            since: Utc::now(),
            generated_at: Utc::now(),
            repositories: vec![Repository {
                path: PathBuf::from("/test/repo"),
                name: "test".to_string(),
                default_branch: "main".to_string(),
                branches: vec![],
            }],
            todos: vec![],
            notes: vec![],
        };
        assert!(chronicle_with_repos.has_activity());

        let chronicle_with_todos = Chronicle {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            since: Utc::now(),
            generated_at: Utc::now(),
            repositories: vec![],
            todos: vec![Todo {
                content: "Task".to_string(),
                status: TodoStatus::Pending,
                change: ChangeKind::New,
                previous_status: None,
                file: PathBuf::from("todo.txt"),
                line: 1,
            }],
            notes: vec![],
        };
        assert!(chronicle_with_todos.has_activity());
    }
}
