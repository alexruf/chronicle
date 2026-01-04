use chrono::{Local, NaiveDate, Utc};
use std::fs;
use std::path::PathBuf;

use crate::collectors::{GitCollector, NotesCollector, TodoCollector};
use crate::config;
use crate::error::Result;
use crate::models::Chronicle;
use crate::renderer::Renderer;
use crate::state;

/// Generate a daily chronicle
pub fn run(
    config_path: Option<PathBuf>,
    date: Option<String>,
    since: Option<String>,
    only: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let config_path = config_path.unwrap_or_else(|| PathBuf::from("chronicle.toml"));

    // Load configuration
    let config = config::load(&config_path)?;

    // Load state
    let mut state = state::load(&config.state_file)?;

    // Parse date (default to today)
    let chronicle_date = if let Some(date_str) = date {
        NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
            crate::error::ChronicleError::Config(format!("Invalid date format: {}", e))
        })?
    } else {
        Local::now().date_naive()
    };

    // Parse since timestamp
    let since_time = if let Some(since_str) = since {
        chrono::DateTime::parse_from_rfc3339(&since_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                crate::error::ChronicleError::Config(format!("Invalid since timestamp: {}", e))
            })?
    } else {
        // Default to 24 hours ago
        Utc::now() - chrono::Duration::hours(24)
    };

    // Determine which collectors to run
    let run_git = only.as_deref().is_none_or(|s| s.contains("git"));
    let run_todos = only.as_deref().is_none_or(|s| s.contains("todos"));
    let run_notes = only.as_deref().is_none_or(|s| s.contains("notes"));

    // Run collectors
    let repositories = if run_git {
        let collector = GitCollector::new(&config);
        collector.collect(&mut state, since_time)?
    } else {
        vec![]
    };

    let todos = if run_todos {
        let collector = TodoCollector::new(&config);
        collector.collect(&mut state)?
    } else {
        vec![]
    };

    let notes = if run_notes {
        let collector = NotesCollector::new(&config);
        collector.collect(&mut state, since_time)?
    } else {
        vec![]
    };

    // Build chronicle
    let chronicle = Chronicle {
        date: chronicle_date,
        since: since_time,
        generated_at: Utc::now(),
        repositories,
        todos,
        notes,
    };

    // Check if there's any activity
    if !chronicle.has_activity() {
        println!("No activity to report.");
        return Ok(());
    }

    // Render to Markdown
    let renderer = Renderer::new(&config);
    let markdown = renderer.render(&chronicle);

    if dry_run {
        // Print to stdout with rich terminal formatting (if supported)
        crate::display::print_markdown(&markdown);
    } else {
        // Write to file
        let filename = format!("chronicle-{}.md", chronicle_date.format("%Y-%m-%d"));
        let output_path = config.output_dir.join(filename);

        // Ensure output directory exists
        if !config.output_dir.exists() {
            fs::create_dir_all(&config.output_dir)?;
        }

        fs::write(&output_path, markdown)?;

        println!("Chronicle written to: {}", output_path.display());

        // Save state
        state::save(&state, &config.state_file)?;
    }

    Ok(())
}
