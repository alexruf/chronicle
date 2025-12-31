use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{ChronicleError, Result};
use crate::models::{ChangeKind, Todo, TodoStatus};
use crate::state::{self, SourceState, State};

/// TODO collector for parsing TODO/Inbox markdown files
pub struct TodoCollector<'a> {
    config: &'a Config,
}

impl<'a> TodoCollector<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Collect TODOs from all configured files
    pub fn collect(&self, state: &mut State) -> Result<Vec<Todo>> {
        let mut all_todos = Vec::new();

        for todo_file in &self.config.todo_files {
            match self.collect_file(todo_file, state) {
                Ok(todos) => {
                    all_todos.extend(todos);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Skipping TODO file '{}': {}",
                        todo_file.display(),
                        e
                    );
                }
            }
        }

        Ok(all_todos)
    }

    /// Collect TODOs from a single file
    fn collect_file(&self, file_path: &Path, state: &mut State) -> Result<Vec<Todo>> {
        // Read file content
        let content = fs::read_to_string(file_path).map_err(|e| {
            ChronicleError::Collector(format!(
                "Cannot read TODO file '{}': {}",
                file_path.display(),
                e
            ))
        })?;

        // Get file metadata for last modified time
        let _metadata = fs::metadata(file_path)?;

        // Parse todos from content
        let mut todos = self.parse_todos(&content, file_path)?;

        // Detect changes using state
        self.detect_changes(&mut todos, state, file_path);

        // Update state with all TODOs (before filtering)
        self.update_state_for_file(state, file_path, &todos);

        // Filter out unchanged todos
        let changed_todos: Vec<Todo> = todos
            .into_iter()
            .filter(|t| t.change != ChangeKind::Unchanged)
            .collect();

        Ok(changed_todos)
    }

    /// Parse TODO items from file content
    fn parse_todos(&self, content: &str, file_path: &Path) -> Result<Vec<Todo>> {
        let mut todos = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check for TODO patterns
            if let Some(todo) = self.parse_todo_line(trimmed, file_path, line_num + 1)? {
                todos.push(todo);
            }
        }

        Ok(todos)
    }

    /// Parse a single TODO line
    fn parse_todo_line(
        &self,
        line: &str,
        file_path: &Path,
        line_num: usize,
    ) -> Result<Option<Todo>> {
        let (status, content) = if let Some(content) = line.strip_prefix("- [ ] ") {
            (TodoStatus::Pending, content)
        } else if let Some(content) = line.strip_prefix("- [x] ") {
            (TodoStatus::Done, content)
        } else if let Some(content) = line.strip_prefix("- [~] ") {
            (TodoStatus::InProgress, content)
        } else {
            return Ok(None);
        };

        let content = content.to_string();

        Ok(Some(Todo {
            content,
            status,
            change: ChangeKind::New, // Will be updated by detect_changes
            previous_status: None,
            file: file_path.to_path_buf(),
            line: line_num,
        }))
    }

    /// Detect changes in TODOs compared to state
    fn detect_changes(&self, todos: &mut [Todo], state: &State, file_path: &Path) {
        let source_key = file_path.to_string_lossy().to_string();
        let source_state = state::get_source(state, &source_key);

        // Get previous TODO hashes if available
        let previous_hashes = match source_state {
            Some(SourceState::Todo { item_hashes, .. }) => Some(item_hashes),
            _ => None,
        };

        if let Some(prev_hashes) = previous_hashes {
            // Build map of previous todos by hash
            let prev_map: HashMap<String, ()> =
                prev_hashes.iter().map(|h| (h.clone(), ())).collect();

            for todo in todos.iter_mut() {
                let hash = self.hash_todo(todo);

                if prev_map.contains_key(&hash) {
                    // TODO exists unchanged
                    todo.change = ChangeKind::Unchanged;
                } else {
                    // Check if content exists but status changed
                    let content_hash = self.hash_todo_content(&todo.content, &todo.file, todo.line);
                    let mut found_previous_status = false;

                    for prev_hash in prev_hashes {
                        // Check if this hash contains the same content (after the status prefix)
                        // Hash format is "Status:file:line:content"
                        if prev_hash.contains(&content_hash) {
                            // This is the same TODO but with different status
                            todo.change = ChangeKind::Modified;
                            // Try to extract previous status
                            if let Some(status) = self.extract_status_from_hash(prev_hash) {
                                todo.previous_status = Some(status);
                            }
                            found_previous_status = true;
                            break;
                        }
                    }

                    if !found_previous_status {
                        todo.change = ChangeKind::New;
                    }
                }
            }
        } else {
            // No previous state, all TODOs are new
            for todo in todos.iter_mut() {
                todo.change = ChangeKind::New;
            }
        }
    }

    /// Generate hash for a TODO item (content + status + location)
    fn hash_todo(&self, todo: &Todo) -> String {
        format!(
            "{:?}:{}:{}:{}",
            todo.status,
            todo.file.display(),
            todo.line,
            todo.content
        )
    }

    /// Generate hash for TODO content only (for detecting status changes)
    fn hash_todo_content(&self, content: &str, file: &Path, line: usize) -> String {
        format!("{}:{}:{}", file.display(), line, content)
    }

    /// Extract status from hash string
    fn extract_status_from_hash(&self, hash: &str) -> Option<TodoStatus> {
        if hash.starts_with("Pending:") {
            Some(TodoStatus::Pending)
        } else if hash.starts_with("Done:") {
            Some(TodoStatus::Done)
        } else if hash.starts_with("InProgress:") {
            Some(TodoStatus::InProgress)
        } else {
            None
        }
    }

    /// Update state for a single file with its TODOs
    fn update_state_for_file(&self, state: &mut State, file_path: &Path, todos: &[Todo]) {
        let source_key = file_path.to_string_lossy().to_string();

        let item_hashes: Vec<String> = todos.iter().map(|t| self.hash_todo(t)).collect();

        let source_state = SourceState::Todo {
            last_checked: Utc::now(),
            last_modified: Utc::now(),
            item_hashes,
        };

        state::update_source(state, source_key, source_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_todo_line_pending() {
        let config = Config::default();
        let collector = TodoCollector::new(&config);

        let todo = collector
            .parse_todo_line("- [ ] Buy milk", Path::new("todo.md"), 1)
            .unwrap()
            .unwrap();

        assert_eq!(todo.content, "Buy milk");
        assert_eq!(todo.status, TodoStatus::Pending);
        assert_eq!(todo.line, 1);
    }

    #[test]
    fn test_parse_todo_line_done() {
        let config = Config::default();
        let collector = TodoCollector::new(&config);

        let todo = collector
            .parse_todo_line("- [x] Buy milk", Path::new("todo.md"), 1)
            .unwrap()
            .unwrap();

        assert_eq!(todo.content, "Buy milk");
        assert_eq!(todo.status, TodoStatus::Done);
    }

    #[test]
    fn test_parse_todo_line_in_progress() {
        let config = Config::default();
        let collector = TodoCollector::new(&config);

        let todo = collector
            .parse_todo_line("- [~] Working on it", Path::new("todo.md"), 1)
            .unwrap()
            .unwrap();

        assert_eq!(todo.content, "Working on it");
        assert_eq!(todo.status, TodoStatus::InProgress);
    }

    #[test]
    fn test_parse_todo_line_not_todo() {
        let config = Config::default();
        let collector = TodoCollector::new(&config);

        let result = collector
            .parse_todo_line("Just a regular line", Path::new("todo.md"), 1)
            .unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_todos() {
        let config = Config::default();
        let collector = TodoCollector::new(&config);

        let content = r#"
# My TODO List

- [ ] Task 1
- [x] Task 2
- [~] Task 3

Some other text

- [ ] Task 4
"#;

        let todos = collector
            .parse_todos(content, Path::new("todo.md"))
            .unwrap();

        assert_eq!(todos.len(), 4);
        assert_eq!(todos[0].content, "Task 1");
        assert_eq!(todos[1].content, "Task 2");
        assert_eq!(todos[2].content, "Task 3");
        assert_eq!(todos[3].content, "Task 4");
    }

    #[test]
    fn test_collect_from_empty_config() {
        let config = Config::default();
        let collector = TodoCollector::new(&config);
        let mut state = State::default();

        let result = collector.collect(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_collect_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let todo_file = temp_dir.path().join("todo.md");

        fs::write(
            &todo_file,
            r#"
- [ ] New task
- [x] Done task
"#,
        )
        .unwrap();

        let mut config = Config::default();
        config.todo_files.push(todo_file);

        let collector = TodoCollector::new(&config);
        let mut state = State::default();

        let todos = collector.collect(&mut state).unwrap();

        // All should be marked as new on first run
        assert_eq!(todos.len(), 2);
        assert!(todos.iter().all(|t| t.change == ChangeKind::New));
    }

    #[test]
    fn test_detect_status_change() {
        let temp_dir = TempDir::new().unwrap();
        let todo_file = temp_dir.path().join("todo.md");

        fs::write(&todo_file, "- [ ] Task\n").unwrap();

        let mut config = Config::default();
        config.todo_files.push(todo_file.clone());

        let collector = TodoCollector::new(&config);
        let mut state = State::default();

        // First collection (state is updated automatically)
        let todos = collector.collect(&mut state).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].change, ChangeKind::New);

        // Change status to done
        fs::write(&todo_file, "- [x] Task\n").unwrap();

        // Second collection
        let todos2 = collector.collect(&mut state).unwrap();
        assert_eq!(todos2.len(), 1);
        assert_eq!(todos2[0].change, ChangeKind::Modified);
        assert_eq!(todos2[0].status, TodoStatus::Done);
        assert_eq!(todos2[0].previous_status, Some(TodoStatus::Pending));
    }
}
