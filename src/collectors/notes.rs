use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::Config;
use crate::error::{ChronicleError, Result};
use crate::models::{ChangeKind, Note};
use crate::state::{self, SourceState, State};

/// Notes collector for scanning note directories
pub struct NotesCollector<'a> {
    config: &'a Config,
}

impl<'a> NotesCollector<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Collect notes from all configured directories
    pub fn collect(&self, state: &mut State, since: DateTime<Utc>) -> Result<Vec<Note>> {
        let mut all_notes = Vec::new();

        for notes_dir in &self.config.notes_dirs {
            match self.collect_directory(notes_dir, state, since) {
                Ok(notes) => {
                    all_notes.extend(notes);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Skipping notes directory '{}': {}",
                        notes_dir.display(),
                        e
                    );
                }
            }
        }

        // Sort by modification time (newest first)
        all_notes.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

        // Apply limit
        all_notes.truncate(self.config.limits.max_note_files);

        Ok(all_notes)
    }

    /// Collect notes from a single directory
    fn collect_directory(
        &self,
        dir_path: &Path,
        state: &mut State,
        since: DateTime<Utc>,
    ) -> Result<Vec<Note>> {
        if !dir_path.exists() {
            return Err(ChronicleError::Collector(format!(
                "Notes directory does not exist: {}",
                dir_path.display()
            )));
        }

        if !dir_path.is_dir() {
            return Err(ChronicleError::Collector(format!(
                "Notes path is not a directory: {}",
                dir_path.display()
            )));
        }

        let mut notes = Vec::new();

        // Walk directory (max depth 1 - no recursion)
        for entry in WalkDir::new(dir_path)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Check if file is a markdown file
            if !self.is_markdown_file(path) {
                continue;
            }

            // Get file metadata
            let metadata = match fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let modified = match metadata.modified() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let modified_dt: DateTime<Utc> = modified.into();

            // Check if modified after since time
            if modified_dt < since {
                continue;
            }

            // Determine if note is new or modified
            let change = self.determine_note_change(path, state, dir_path);

            // Extract excerpt
            let excerpt = self.extract_excerpt(path)?;

            notes.push(Note {
                path: path.to_path_buf(),
                change,
                modified_at: modified_dt,
                excerpt,
            });
        }

        // Update state
        self.update_state(state, dir_path, &notes);

        Ok(notes)
    }

    /// Check if a file is a markdown file
    fn is_markdown_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            ext_str == "md" || ext_str == "markdown"
        } else {
            false
        }
    }

    /// Extract excerpt from a note file
    fn extract_excerpt(&self, path: &Path) -> Result<String> {
        let content = fs::read_to_string(path).map_err(|e| {
            ChronicleError::Collector(format!("Cannot read note file '{}': {}", path.display(), e))
        })?;

        // Take up to max_chars_per_item characters
        let max_chars = self.config.limits.max_chars_per_item;
        let excerpt = if content.len() <= max_chars {
            content
        } else {
            // Try to find a sentence boundary
            let truncated = &content[..max_chars];
            if let Some(pos) = truncated.rfind('.') {
                truncated[..=pos].to_string()
            } else if let Some(pos) = truncated.rfind('\n') {
                truncated[..pos].to_string()
            } else {
                format!("{}...", truncated)
            }
        };

        Ok(excerpt.trim().to_string())
    }

    /// Determine if a note is new or modified
    fn determine_note_change(&self, path: &Path, state: &State, dir_path: &Path) -> ChangeKind {
        let source_key = dir_path.to_string_lossy().to_string();
        let source_state = state::get_source(state, &source_key);

        match source_state {
            Some(SourceState::Notes { files, .. }) => {
                let file_key = path.to_string_lossy().to_string();
                if files.contains_key(&file_key) {
                    ChangeKind::Modified
                } else {
                    ChangeKind::New
                }
            }
            _ => ChangeKind::New,
        }
    }

    /// Update state with current notes
    fn update_state(&self, state: &mut State, dir_path: &Path, notes: &[Note]) {
        let source_key = dir_path.to_string_lossy().to_string();

        let mut files = HashMap::new();
        for note in notes {
            let file_key = note.path.to_string_lossy().to_string();
            files.insert(file_key, note.modified_at);
        }

        let source_state = SourceState::Notes {
            last_checked: Utc::now(),
            files,
        };

        state::update_source(state, source_key, source_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_is_markdown_file() {
        let config = Config::default();
        let collector = NotesCollector::new(&config);

        assert!(collector.is_markdown_file(Path::new("test.md")));
        assert!(collector.is_markdown_file(Path::new("test.markdown")));
        assert!(collector.is_markdown_file(Path::new("test.MD")));
        assert!(!collector.is_markdown_file(Path::new("test.txt")));
        assert!(!collector.is_markdown_file(Path::new("test")));
    }

    #[test]
    fn test_extract_excerpt_short() {
        let temp_dir = TempDir::new().unwrap();
        let note_file = temp_dir.path().join("note.md");

        let content = "This is a short note.";
        fs::write(&note_file, content).unwrap();

        let config = Config::default();
        let collector = NotesCollector::new(&config);

        let excerpt = collector.extract_excerpt(&note_file).unwrap();
        assert_eq!(excerpt, content);
    }

    #[test]
    fn test_extract_excerpt_long() {
        let temp_dir = TempDir::new().unwrap();
        let note_file = temp_dir.path().join("note.md");

        // Create content longer than max_chars_per_item (2000)
        let long_content = "a".repeat(3000);
        fs::write(&note_file, &long_content).unwrap();

        let config = Config::default();
        let collector = NotesCollector::new(&config);

        let excerpt = collector.extract_excerpt(&note_file).unwrap();
        assert!(excerpt.len() <= config.limits.max_chars_per_item + 3); // +3 for "..."
    }

    #[test]
    fn test_extract_excerpt_with_sentence() {
        let temp_dir = TempDir::new().unwrap();
        let note_file = temp_dir.path().join("note.md");

        // Create content with sentences
        let content = format!("First sentence. Second sentence. {}", "x".repeat(2000));
        fs::write(&note_file, &content).unwrap();

        let config = Config::default();
        let collector = NotesCollector::new(&config);

        let excerpt = collector.extract_excerpt(&note_file).unwrap();
        assert!(excerpt.ends_with('.'));
    }

    #[test]
    fn test_collect_from_empty_config() {
        let config = Config::default();
        let collector = NotesCollector::new(&config);
        let mut state = State::default();
        let since = Utc::now() - chrono::Duration::hours(24);

        let result = collector.collect(&mut state, since);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_collect_directory_with_notes() {
        let temp_dir = TempDir::new().unwrap();
        let notes_dir = temp_dir.path().to_path_buf();

        // Create some note files
        fs::write(notes_dir.join("note1.md"), "First note content.").unwrap();
        thread::sleep(Duration::from_millis(10));
        fs::write(notes_dir.join("note2.md"), "Second note content.").unwrap();
        fs::write(notes_dir.join("readme.txt"), "Not a markdown file.").unwrap();

        let mut config = Config::default();
        config.notes_dirs.push(notes_dir);

        let collector = NotesCollector::new(&config);
        let mut state = State::default();
        let since = Utc::now() - chrono::Duration::hours(1);

        let notes = collector.collect(&mut state, since).unwrap();

        // Should have 2 markdown files
        assert_eq!(notes.len(), 2);
        // Should be sorted by modification time (newest first)
        assert!(notes[0].modified_at >= notes[1].modified_at);
        // All should be marked as new on first run
        assert!(notes.iter().all(|n| n.change == ChangeKind::New));
    }

    #[test]
    fn test_detect_modified_note() {
        let temp_dir = TempDir::new().unwrap();
        let notes_dir = temp_dir.path().to_path_buf();

        fs::write(notes_dir.join("note.md"), "Initial content.").unwrap();

        let mut config = Config::default();
        config.notes_dirs.push(notes_dir.clone());

        let collector = NotesCollector::new(&config);
        let mut state = State::default();
        let since = Utc::now() - chrono::Duration::hours(1);

        // First collection
        let notes = collector.collect(&mut state, since).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].change, ChangeKind::New);

        // Modify the note
        thread::sleep(Duration::from_millis(10));
        fs::write(notes_dir.join("note.md"), "Modified content.").unwrap();

        // Second collection
        let notes2 = collector.collect(&mut state, since).unwrap();
        assert_eq!(notes2.len(), 1);
        assert_eq!(notes2[0].change, ChangeKind::Modified);
    }

    #[test]
    fn test_respects_max_note_files_limit() {
        let temp_dir = TempDir::new().unwrap();
        let notes_dir = temp_dir.path().to_path_buf();

        // Create more notes than the limit
        for i in 0..50 {
            fs::write(notes_dir.join(format!("note{}.md", i)), "Content.").unwrap();
            thread::sleep(Duration::from_millis(1));
        }

        let mut config = Config::default();
        config.notes_dirs.push(notes_dir);
        config.limits.max_note_files = 30;

        let collector = NotesCollector::new(&config);
        let mut state = State::default();
        let since = Utc::now() - chrono::Duration::hours(1);

        let notes = collector.collect(&mut state, since).unwrap();

        assert_eq!(notes.len(), 30);
    }
}
