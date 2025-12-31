use chrono::{DateTime, TimeZone, Utc};
use git2::{BranchType, Oid, Repository as Git2Repository};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{ChronicleError, Result};
use crate::models::{Branch, ChangeKind, Commit, Repository};
use crate::state::{self, BranchState, SourceState, State};

/// Git collector for extracting commits and branch information
pub struct GitCollector<'a> {
    config: &'a Config,
}

impl<'a> GitCollector<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Collect Git activity from all configured repositories
    pub fn collect(&self, state: &mut State, since: DateTime<Utc>) -> Result<Vec<Repository>> {
        let mut repositories = Vec::new();

        for repo_path in &self.config.repos {
            match self.collect_repository(repo_path, state, since) {
                Ok(Some(repo)) => repositories.push(repo),
                Ok(None) => {
                    // No activity in this repository
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Skipping repository '{}': {}",
                        repo_path.display(),
                        e
                    );
                }
            }
        }

        Ok(repositories)
    }

    /// Collect activity from a single repository
    fn collect_repository(
        &self,
        repo_path: &Path,
        state: &mut State,
        since: DateTime<Utc>,
    ) -> Result<Option<Repository>> {
        let git_repo = self.open_repository(repo_path)?;
        let repo_name = repo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Get default branch (HEAD reference)
        let head = git_repo.head().map_err(|e| {
            ChronicleError::Git(git2::Error::from_str(&format!(
                "Failed to get HEAD for {}: {}",
                repo_path.display(),
                e
            )))
        })?;

        let default_branch = if head.is_branch() {
            head.shorthand().unwrap_or("main").to_string()
        } else {
            "main".to_string()
        };

        // Collect branches with commits
        let branches = self.collect_branches(&git_repo, &default_branch, state, since, repo_path)?;

        // Filter out branches with no commits
        let branches: Vec<Branch> = branches.into_iter().filter(|b| !b.commits.is_empty()).collect();

        if branches.is_empty() {
            return Ok(None);
        }

        // Update state
        self.update_state(state, repo_path, &default_branch, &branches);

        Ok(Some(Repository {
            path: repo_path.to_path_buf(),
            name: repo_name,
            default_branch,
            branches,
        }))
    }

    /// Open a Git repository
    fn open_repository(&self, path: &Path) -> Result<Git2Repository> {
        Git2Repository::open(path).map_err(|e| {
            ChronicleError::Collector(format!(
                "Cannot open Git repository at '{}': {}",
                path.display(),
                e
            ))
        })
    }

    /// Collect branches and their commits
    fn collect_branches(
        &self,
        repo: &Git2Repository,
        default_branch: &str,
        state: &State,
        since: DateTime<Utc>,
        repo_path: &Path,
    ) -> Result<Vec<Branch>> {
        let mut branches = Vec::new();

        // Get source state for this repository
        let source_key = repo_path.to_string_lossy().to_string();
        let source_state = state::get_source(state, &source_key);

        // Get branch states if available
        let branch_states = match source_state {
            Some(SourceState::Git { branches, .. }) => Some(branches),
            _ => None,
        };

        // Iterate through all local branches
        let git_branches = repo.branches(Some(BranchType::Local)).map_err(|e| {
            ChronicleError::Collector(format!("Failed to list branches: {}", e))
        })?;

        for branch_result in git_branches {
            let (branch, _) = branch_result.map_err(|e| {
                ChronicleError::Collector(format!("Failed to get branch: {}", e))
            })?;

            let branch_name = branch
                .name()
                .map_err(|e| ChronicleError::Collector(format!("Failed to get branch name: {}", e)))?
                .unwrap_or("unknown")
                .to_string();

            // Get branch commit
            let branch_ref = branch.get();
            let branch_oid = branch_ref.target().ok_or_else(|| {
                ChronicleError::Collector(format!("Branch {} has no target", branch_name))
            })?;

            // Collect commits for this branch
            let commits = self.collect_commits(repo, branch_oid, since)?;

            if commits.is_empty() && branch_name != default_branch {
                // Skip branches with no new commits (except default branch)
                continue;
            }

            // Determine if this is a new branch
            let change = self.determine_branch_change(&branch_name, branch_states);

            // Calculate ahead/behind relative to default branch
            let (ahead, behind) = if branch_name != default_branch {
                self.calculate_ahead_behind(repo, default_branch, &branch_name)?
            } else {
                (0, 0)
            };

            branches.push(Branch {
                name: branch_name,
                change,
                ahead,
                behind,
                commits,
            });
        }

        Ok(branches)
    }

    /// Collect commits from a branch since a specific time
    fn collect_commits(
        &self,
        repo: &Git2Repository,
        branch_oid: Oid,
        since: DateTime<Utc>,
    ) -> Result<Vec<Commit>> {
        let mut revwalk = repo.revwalk().map_err(|e| {
            ChronicleError::Collector(format!("Failed to create revwalk: {}", e))
        })?;

        revwalk.push(branch_oid).map_err(|e| {
            ChronicleError::Collector(format!("Failed to push branch to revwalk: {}", e))
        })?;

        let mut commits = Vec::new();
        let mut seen_files = HashSet::new();

        for oid_result in revwalk {
            if commits.len() >= self.config.limits.max_commits {
                break;
            }

            let oid = oid_result.map_err(|e| {
                ChronicleError::Collector(format!("Failed to get commit OID: {}", e))
            })?;

            let git_commit = repo.find_commit(oid).map_err(|e| {
                ChronicleError::Collector(format!("Failed to find commit: {}", e))
            })?;

            // Check if commit is within time range
            let commit_time = Utc.timestamp_opt(git_commit.time().seconds(), 0).single()
                .ok_or_else(|| {
                    ChronicleError::Collector("Invalid commit timestamp".to_string())
                })?;

            if commit_time < since {
                break;
            }

            // Extract commit information
            let hash = format!("{:.7}", oid);
            let message = git_commit
                .message()
                .unwrap_or("(no message)")
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(72)
                .collect();

            let author = git_commit.author().name().unwrap_or("Unknown").to_string();

            // Collect changed files
            let files = self.collect_commit_files(repo, &git_commit, &mut seen_files)?;

            commits.push(Commit {
                hash,
                message,
                author,
                timestamp: commit_time,
                files,
            });
        }

        Ok(commits)
    }

    /// Collect files changed in a commit
    fn collect_commit_files(
        &self,
        repo: &Git2Repository,
        commit: &git2::Commit,
        seen_files: &mut HashSet<PathBuf>,
    ) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let commit_tree = commit.tree().map_err(|e| {
            ChronicleError::Collector(format!("Failed to get commit tree: {}", e))
        })?;

        let parent_tree = commit
            .parent(0)
            .ok()
            .and_then(|p| p.tree().ok());

        let diff = repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)
            .map_err(|e| {
                ChronicleError::Collector(format!("Failed to create diff: {}", e))
            })?;

        diff.foreach(
            &mut |delta, _| {
                if seen_files.len() >= self.config.limits.max_changed_files {
                    return true; // Stop iterating
                }

                if let Some(path) = delta.new_file().path() {
                    let path_buf = path.to_path_buf();
                    if seen_files.insert(path_buf.clone()) {
                        files.push(path_buf);
                    }
                }
                true
            },
            None,
            None,
            None,
        )
        .map_err(|e| {
            ChronicleError::Collector(format!("Failed to iterate diff: {}", e))
        })?;

        Ok(files)
    }

    /// Calculate commits ahead and behind between two branches
    fn calculate_ahead_behind(
        &self,
        repo: &Git2Repository,
        base_branch: &str,
        compare_branch: &str,
    ) -> Result<(usize, usize)> {
        // Get OIDs for both branches
        let base_ref = repo
            .find_branch(base_branch, BranchType::Local)
            .map_err(|e| {
                ChronicleError::Collector(format!("Failed to find base branch {}: {}", base_branch, e))
            })?;
        let base_oid = base_ref.get().target().ok_or_else(|| {
            ChronicleError::Collector(format!("Base branch {} has no target", base_branch))
        })?;

        let compare_ref = repo
            .find_branch(compare_branch, BranchType::Local)
            .map_err(|e| {
                ChronicleError::Collector(format!(
                    "Failed to find compare branch {}: {}",
                    compare_branch, e
                ))
            })?;
        let compare_oid = compare_ref.get().target().ok_or_else(|| {
            ChronicleError::Collector(format!("Compare branch {} has no target", compare_branch))
        })?;

        // Calculate ahead/behind
        let (ahead, behind) = repo.graph_ahead_behind(compare_oid, base_oid).map_err(|e| {
            ChronicleError::Collector(format!("Failed to calculate ahead/behind: {}", e))
        })?;

        Ok((ahead, behind))
    }

    /// Determine if a branch is new or modified
    fn determine_branch_change(
        &self,
        branch_name: &str,
        branch_states: Option<&HashMap<String, BranchState>>,
    ) -> ChangeKind {
        match branch_states {
            Some(states) => {
                if states.contains_key(branch_name) {
                    ChangeKind::Modified
                } else {
                    ChangeKind::New
                }
            }
            None => ChangeKind::New,
        }
    }

    /// Update state with latest branch information
    fn update_state(
        &self,
        state: &mut State,
        repo_path: &Path,
        default_branch: &str,
        branches: &[Branch],
    ) {
        let source_key = repo_path.to_string_lossy().to_string();

        // Build branch states map
        let mut branch_states = HashMap::new();
        for branch in branches {
            let last_commit = branch
                .commits
                .first()
                .map(|c| c.hash.clone())
                .unwrap_or_default();

            let first_seen = if branch.change == ChangeKind::New {
                Some(Utc::now())
            } else {
                // Try to preserve existing first_seen
                None
            };

            branch_states.insert(
                branch.name.clone(),
                BranchState {
                    last_commit,
                    last_seen: Utc::now(),
                    first_seen,
                },
            );
        }

        let source_state = SourceState::Git {
            last_checked: Utc::now(),
            default_branch: default_branch.to_string(),
            branches: branch_states,
        };

        state::update_source(state, source_key, source_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Display, Limits};
    use std::process::Command;
    use tempfile::TempDir;

    /// Helper to create a test Git repository
    fn create_test_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize Git repo
        Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Configure Git
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Create initial commit
        std::fs::write(repo_path.join("test.txt"), "initial content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        (temp_dir, repo_path)
    }

    #[test]
    fn test_open_repository() {
        let (_temp_dir, repo_path) = create_test_repo();

        let config = Config::default();
        let collector = GitCollector::new(&config);

        let result = collector.open_repository(&repo_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_invalid_repository() {
        let config = Config::default();
        let collector = GitCollector::new(&config);

        let result = collector.open_repository(Path::new("/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_collect_from_empty_config() {
        let config = Config::default();
        let collector = GitCollector::new(&config);
        let mut state = State::default();
        let since = Utc::now();

        let result = collector.collect(&mut state, since);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_collect_repository_with_commits() {
        let (_temp_dir, repo_path) = create_test_repo();

        let mut config = Config::default();
        config.repos.push(repo_path.clone());

        let collector = GitCollector::new(&config);
        let mut state = State::default();
        let since = Utc::now() - chrono::Duration::hours(1);

        let result = collector.collect(&mut state, since);
        assert!(result.is_ok());

        let repos = result.unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].branches.len(), 1);
        assert!(!repos[0].branches[0].commits.is_empty());
    }
}
