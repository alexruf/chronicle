use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Indicates whether an item is new, modified, or unchanged
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeKind {
    New,
    Modified,
    Unchanged,
}

// ============================================================================
// Git Models
// ============================================================================

/// A single Git commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    /// Short commit hash (7 characters)
    pub hash: String,
    /// Commit message (first line, max 72 chars)
    pub message: String,
    /// Commit author name
    pub author: String,
    /// Commit timestamp
    pub timestamp: DateTime<Utc>,
    /// List of files changed in this commit
    pub files: Vec<PathBuf>,
}

/// A Git branch with its commits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Branch name
    pub name: String,
    /// Whether this branch is new, modified, or unchanged
    pub change: ChangeKind,
    /// Commits ahead of default branch
    pub ahead: usize,
    /// Commits behind default branch
    pub behind: usize,
    /// List of commits on this branch
    pub commits: Vec<Commit>,
}

/// A Git repository with its branches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Path to the repository
    pub path: PathBuf,
    /// Repository name (derived from path)
    pub name: String,
    /// Default branch name (usually "main" or "master")
    pub default_branch: String,
    /// All branches with commits
    pub branches: Vec<Branch>,
}

impl Repository {
    /// Total number of commits across all branches
    pub fn commit_count(&self) -> usize {
        self.branches.iter().map(|b| b.commits.len()).sum()
    }

    /// Total number of unique files changed across all commits
    #[allow(dead_code)]
    pub fn files_changed(&self) -> usize {
        let mut files = std::collections::HashSet::new();
        for branch in &self.branches {
            for commit in &branch.commits {
                for file in &commit.files {
                    files.insert(file);
                }
            }
        }
        files.len()
    }

    /// Number of new branches
    pub fn new_branch_count(&self) -> usize {
        self.branches
            .iter()
            .filter(|b| b.change == ChangeKind::New)
            .count()
    }
}

// ============================================================================
// TODO Models
// ============================================================================

/// Status of a TODO item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TodoStatus {
    /// Pending: - [ ]
    Pending,
    /// Done: - [x]
    Done,
    /// In Progress: - [~]
    InProgress,
}

/// A TODO item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    /// TODO content text
    pub content: String,
    /// Current status
    pub status: TodoStatus,
    /// Whether this TODO is new, modified, or unchanged
    pub change: ChangeKind,
    /// Previous status (for change detection)
    pub previous_status: Option<TodoStatus>,
    /// Source file path
    pub file: PathBuf,
    /// Line number in file
    pub line: usize,
}

impl Todo {
    /// Check if this TODO was just completed
    pub fn was_completed(&self) -> bool {
        self.status == TodoStatus::Done
            && self.previous_status.is_some()
            && self.previous_status != Some(TodoStatus::Done)
    }
}

// ============================================================================
// Notes Models
// ============================================================================

/// A note file update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Path to the note file
    pub path: PathBuf,
    /// Whether this note is new or modified
    pub change: ChangeKind,
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
    /// Excerpt from the note (respects max_chars_per_item limit)
    pub excerpt: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_commit_count() {
        let repo = Repository {
            path: PathBuf::from("/test/repo"),
            name: "test-repo".to_string(),
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
                            message: "First commit".to_string(),
                            author: "Test Author".to_string(),
                            timestamp: Utc::now(),
                            files: vec![],
                        },
                        Commit {
                            hash: "def5678".to_string(),
                            message: "Second commit".to_string(),
                            author: "Test Author".to_string(),
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
                        message: "Feature commit".to_string(),
                        author: "Test Author".to_string(),
                        timestamp: Utc::now(),
                        files: vec![],
                    }],
                },
            ],
        };

        assert_eq!(repo.commit_count(), 3);
    }

    #[test]
    fn test_repository_new_branch_count() {
        let repo = Repository {
            path: PathBuf::from("/test/repo"),
            name: "test-repo".to_string(),
            default_branch: "main".to_string(),
            branches: vec![
                Branch {
                    name: "main".to_string(),
                    change: ChangeKind::Modified,
                    ahead: 0,
                    behind: 0,
                    commits: vec![],
                },
                Branch {
                    name: "feature1".to_string(),
                    change: ChangeKind::New,
                    ahead: 1,
                    behind: 0,
                    commits: vec![],
                },
                Branch {
                    name: "feature2".to_string(),
                    change: ChangeKind::New,
                    ahead: 2,
                    behind: 0,
                    commits: vec![],
                },
            ],
        };

        assert_eq!(repo.new_branch_count(), 2);
    }

    #[test]
    fn test_repository_files_changed() {
        let repo = Repository {
            path: PathBuf::from("/test/repo"),
            name: "test-repo".to_string(),
            default_branch: "main".to_string(),
            branches: vec![Branch {
                name: "main".to_string(),
                change: ChangeKind::Modified,
                ahead: 0,
                behind: 0,
                commits: vec![
                    Commit {
                        hash: "abc1234".to_string(),
                        message: "First commit".to_string(),
                        author: "Test Author".to_string(),
                        timestamp: Utc::now(),
                        files: vec![PathBuf::from("file1.rs"), PathBuf::from("file2.rs")],
                    },
                    Commit {
                        hash: "def5678".to_string(),
                        message: "Second commit".to_string(),
                        author: "Test Author".to_string(),
                        timestamp: Utc::now(),
                        files: vec![PathBuf::from("file2.rs"), PathBuf::from("file3.rs")],
                    },
                ],
            }],
        };

        assert_eq!(repo.files_changed(), 3);
    }

    #[test]
    fn test_todo_was_completed() {
        let completed_todo = Todo {
            content: "Test task".to_string(),
            status: TodoStatus::Done,
            change: ChangeKind::Modified,
            previous_status: Some(TodoStatus::Pending),
            file: PathBuf::from("todo.txt"),
            line: 1,
        };
        assert!(completed_todo.was_completed());

        let already_done_todo = Todo {
            content: "Test task".to_string(),
            status: TodoStatus::Done,
            change: ChangeKind::Unchanged,
            previous_status: Some(TodoStatus::Done),
            file: PathBuf::from("todo.txt"),
            line: 1,
        };
        assert!(!already_done_todo.was_completed());

        let new_done_todo = Todo {
            content: "Test task".to_string(),
            status: TodoStatus::Done,
            change: ChangeKind::New,
            previous_status: None,
            file: PathBuf::from("todo.txt"),
            line: 1,
        };
        assert!(!new_done_todo.was_completed());
    }
}
