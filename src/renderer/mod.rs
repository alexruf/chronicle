//! Markdown renderer module
//!
//! Generates daily chronicle output in Markdown format.
//! Renders sections: Summary, Git Activity, TODOs, Notes.

use chrono::{DateTime, NaiveDate, Utc};

use crate::config::Config;
use crate::models::{Branch, ChangeKind, Chronicle, Note, Repository, Todo, TodoStatus};

/// Markdown renderer for chronicles
pub struct Renderer<'a> {
    config: &'a Config,
}

impl<'a> Renderer<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Render a complete chronicle to Markdown
    pub fn render(&self, chronicle: &Chronicle) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&self.render_header(
            &chronicle.date,
            chronicle.generated_at,
            chronicle.since,
        ));
        output.push_str("\n\n");

        // Summary
        output.push_str(&self.render_summary(chronicle));
        output.push_str("\n\n");

        // Git Activity
        if !chronicle.repositories.is_empty() {
            output.push_str(&self.render_git_activity(&chronicle.repositories));
            output.push_str("\n\n");
        }

        // TODOs
        if !chronicle.todos.is_empty() {
            output.push_str(&self.render_todos(&chronicle.todos));
            output.push_str("\n\n");
        }

        // Notes
        if !chronicle.notes.is_empty() {
            output.push_str(&self.render_notes(&chronicle.notes));
            output.push_str("\n\n");
        }

        output.trim_end().to_string()
    }

    /// Render header section
    fn render_header(
        &self,
        date: &NaiveDate,
        generated_at: DateTime<Utc>,
        since: DateTime<Utc>,
    ) -> String {
        let mut output = String::new();

        output.push_str(&format!("# Chronicle: {}\n\n", date.format("%Y-%m-%d")));
        output.push_str(&format!(
            "**Generated:** {}\n",
            generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!(
            "**Since:** {}",
            since.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        output
    }

    /// Render summary statistics table
    fn render_summary(&self, chronicle: &Chronicle) -> String {
        let stats = chronicle.stats();
        let mut output = String::new();

        output.push_str("## Summary\n\n");
        output.push_str("| Category | Count |\n");
        output.push_str("|----------|-------|\n");
        output.push_str(&format!("| Repositories | {} |\n", stats.repo_count));
        output.push_str(&format!("| Commits | {} |\n", stats.commit_count));
        output.push_str(&format!("| New Branches | {} |\n", stats.new_branch_count));
        output.push_str(&format!("| New TODOs | {} |\n", stats.todos_new));
        output.push_str(&format!(
            "| Completed TODOs | {} |\n",
            stats.todos_completed
        ));
        output.push_str(&format!("| Note Updates | {} |", stats.notes_count));

        output
    }

    /// Render Git activity section
    fn render_git_activity(&self, repositories: &[Repository]) -> String {
        let mut output = String::new();

        output.push_str("## Git Activity\n");

        for repo in repositories {
            output.push('\n');
            output.push_str(&self.render_repository(repo));
        }

        output
    }

    /// Render a single repository
    fn render_repository(&self, repo: &Repository) -> String {
        let mut output = String::new();

        output.push_str(&format!("### {}\n\n", repo.name));
        output.push_str(&format!("**Path:** `{}`\n\n", repo.path.display()));

        // Sort branches: default first, then by commit count
        let mut sorted_branches = repo.branches.clone();
        sorted_branches.sort_by(|a, b| {
            if a.name == repo.default_branch {
                std::cmp::Ordering::Less
            } else if b.name == repo.default_branch {
                std::cmp::Ordering::Greater
            } else {
                b.commits.len().cmp(&a.commits.len())
            }
        });

        for branch in &sorted_branches {
            output.push_str(&self.render_branch(branch, &repo.default_branch));
            output.push('\n');
        }

        output
    }

    /// Render a single branch
    fn render_branch(&self, branch: &Branch, default_branch: &str) -> String {
        let mut output = String::new();

        // Branch header
        let change_marker = match branch.change {
            ChangeKind::New => " ← NEW",
            _ => "",
        };

        let ahead_behind =
            if branch.name != default_branch && (branch.ahead > 0 || branch.behind > 0) {
                format!(" (ahead {}, behind {})", branch.ahead, branch.behind)
            } else {
                String::new()
            };

        output.push_str(&format!(
            "#### `{}`{}{}\n\n",
            branch.name, ahead_behind, change_marker
        ));

        // Commits
        if !branch.commits.is_empty() {
            for commit in &branch.commits {
                let author_info = if self.config.display.show_authors {
                    format!(" — *{}*", commit.author)
                } else {
                    String::new()
                };

                output.push_str(&format!(
                    "- `{}` {}{}  \n",
                    commit.hash, commit.message, author_info
                ));
            }

            // Changed files
            let all_files: std::collections::HashSet<_> =
                branch.commits.iter().flat_map(|c| &c.files).collect();

            if !all_files.is_empty() {
                output.push('\n');
                output.push_str(
                    &self.render_changed_files(&all_files.into_iter().collect::<Vec<_>>()),
                );
            }
        }

        output
    }

    /// Render changed files as collapsible details
    fn render_changed_files(&self, files: &[&std::path::PathBuf]) -> String {
        let mut output = String::new();

        let max_files = self.config.limits.max_changed_files;
        let file_count = files.len();
        let display_count = file_count.min(max_files);

        output.push_str("<details>\n");
        output.push_str(&format!(
            "<summary>Changed files ({})</summary>\n\n",
            file_count
        ));

        for file in files.iter().take(display_count) {
            output.push_str(&format!("- `{}`\n", file.display()));
        }

        if file_count > max_files {
            output.push_str(&format!(
                "\n*... and {} more files*\n",
                file_count - max_files
            ));
        }

        output.push_str("\n</details>\n");

        output
    }

    /// Render TODOs section
    fn render_todos(&self, todos: &[Todo]) -> String {
        let mut output = String::new();

        output.push_str("## TODOs\n");

        // Group by file
        let mut todos_by_file = std::collections::HashMap::new();
        for todo in todos {
            todos_by_file
                .entry(&todo.file)
                .or_insert_with(Vec::new)
                .push(todo);
        }

        for (file, file_todos) in todos_by_file {
            output.push('\n');
            output.push_str(&format!("### `{}`\n\n", file.display()));

            for todo in file_todos {
                output.push_str(&self.render_todo(todo));
            }
        }

        output
    }

    /// Render a single TODO
    fn render_todo(&self, todo: &Todo) -> String {
        let status_marker = match todo.status {
            TodoStatus::Pending => "[ ]",
            TodoStatus::Done => "[x]",
            TodoStatus::InProgress => "[~]",
        };

        let change_marker = match todo.change {
            ChangeKind::New => " ← NEW",
            ChangeKind::Modified if todo.was_completed() => " ← DONE",
            ChangeKind::Modified => " ← MODIFIED",
            ChangeKind::Unchanged => "",
        };

        format!("- {} {}{}  \n", status_marker, todo.content, change_marker)
    }

    /// Render Notes section
    fn render_notes(&self, notes: &[Note]) -> String {
        let mut output = String::new();

        output.push_str("## Notes\n\n");

        for note in notes {
            output.push_str(&self.render_note(note));
            output.push('\n');
        }

        output
    }

    /// Render a single note
    fn render_note(&self, note: &Note) -> String {
        let change_marker = match note.change {
            ChangeKind::New => " ← new",
            ChangeKind::Modified => " ← modified",
            ChangeKind::Unchanged => "",
        };

        let mut output = String::new();
        output.push_str(&format!(
            "### `{}`{}\n\n",
            note.path.display(),
            change_marker
        ));
        output.push_str(&format!(
            "*Modified: {}*\n\n",
            note.modified_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!("{}\n", note.excerpt));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Commit;
    use std::path::PathBuf;

    fn create_test_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_render_header() {
        let config = create_test_config();
        let renderer = Renderer::new(&config);

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let generated_at = Utc::now();
        let since = Utc::now() - chrono::Duration::hours(24);

        let output = renderer.render_header(&date, generated_at, since);

        assert!(output.contains("# Chronicle: 2024-01-15"));
        assert!(output.contains("**Generated:**"));
        assert!(output.contains("**Since:**"));
    }

    #[test]
    fn test_render_summary() {
        let config = create_test_config();
        let renderer = Renderer::new(&config);

        let chronicle = Chronicle {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            since: Utc::now(),
            generated_at: Utc::now(),
            repositories: vec![],
            todos: vec![],
            notes: vec![],
        };

        let output = renderer.render_summary(&chronicle);

        assert!(output.contains("## Summary"));
        assert!(output.contains("| Repositories | 0 |"));
        assert!(output.contains("| Commits | 0 |"));
    }

    #[test]
    fn test_render_todo() {
        let config = create_test_config();
        let renderer = Renderer::new(&config);

        let todo = Todo {
            content: "Buy milk".to_string(),
            status: TodoStatus::Pending,
            change: ChangeKind::New,
            previous_status: None,
            file: PathBuf::from("todo.md"),
            line: 1,
        };

        let output = renderer.render_todo(&todo);

        assert!(output.contains("- [ ] Buy milk"));
        assert!(output.contains("← NEW"));
    }

    #[test]
    fn test_render_todo_completed() {
        let config = create_test_config();
        let renderer = Renderer::new(&config);

        let todo = Todo {
            content: "Buy milk".to_string(),
            status: TodoStatus::Done,
            change: ChangeKind::Modified,
            previous_status: Some(TodoStatus::Pending),
            file: PathBuf::from("todo.md"),
            line: 1,
        };

        let output = renderer.render_todo(&todo);

        assert!(output.contains("- [x] Buy milk"));
        assert!(output.contains("← DONE"));
    }

    #[test]
    fn test_render_note() {
        let config = create_test_config();
        let renderer = Renderer::new(&config);

        let note = Note {
            path: PathBuf::from("notes/idea.md"),
            change: ChangeKind::New,
            modified_at: Utc::now(),
            excerpt: "This is a great idea.".to_string(),
        };

        let output = renderer.render_note(&note);

        assert!(output.contains("### `notes/idea.md`"));
        assert!(output.contains("← new"));
        assert!(output.contains("This is a great idea."));
    }

    #[test]
    fn test_render_branch() {
        let config = create_test_config();
        let renderer = Renderer::new(&config);

        let branch = Branch {
            name: "feature".to_string(),
            change: ChangeKind::New,
            ahead: 2,
            behind: 0,
            commits: vec![Commit {
                hash: "abc1234".to_string(),
                message: "Add feature".to_string(),
                author: "Test Author".to_string(),
                timestamp: Utc::now(),
                files: vec![],
            }],
        };

        let output = renderer.render_branch(&branch, "main");

        assert!(output.contains("#### `feature`"));
        assert!(output.contains("← NEW"));
        assert!(output.contains("ahead 2"));
        assert!(output.contains("`abc1234` Add feature"));
    }

    #[test]
    fn test_render_with_author() {
        let mut config = create_test_config();
        config.display.show_authors = true;
        let renderer = Renderer::new(&config);

        let branch = Branch {
            name: "main".to_string(),
            change: ChangeKind::Modified,
            ahead: 0,
            behind: 0,
            commits: vec![Commit {
                hash: "abc1234".to_string(),
                message: "Fix bug".to_string(),
                author: "Alice".to_string(),
                timestamp: Utc::now(),
                files: vec![],
            }],
        };

        let output = renderer.render_branch(&branch, "main");

        assert!(output.contains("— *Alice*"));
    }

    #[test]
    fn test_render_without_author() {
        let mut config = create_test_config();
        config.display.show_authors = false;
        let renderer = Renderer::new(&config);

        let branch = Branch {
            name: "main".to_string(),
            change: ChangeKind::Modified,
            ahead: 0,
            behind: 0,
            commits: vec![Commit {
                hash: "abc1234".to_string(),
                message: "Fix bug".to_string(),
                author: "Alice".to_string(),
                timestamp: Utc::now(),
                files: vec![],
            }],
        };

        let output = renderer.render_branch(&branch, "main");

        assert!(!output.contains("Alice"));
    }
}
