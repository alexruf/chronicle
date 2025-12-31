use std::fs;
use std::path::PathBuf;

use crate::config;
use crate::error::{ChronicleError, Result};

/// Display the most recent chronicle
pub fn latest(config_path: Option<PathBuf>) -> Result<()> {
    let config_path = config_path.unwrap_or_else(|| PathBuf::from("chronicle.toml"));

    // Load configuration
    let config = config::load(&config_path)?;

    // Find latest chronicle file
    let latest_file = find_latest_chronicle(&config.output_dir)?;

    // Read and display
    let content = fs::read_to_string(&latest_file)?;
    println!("{}", content);

    Ok(())
}

/// Find the most recent chronicle file in the output directory
fn find_latest_chronicle(output_dir: &std::path::Path) -> Result<PathBuf> {
    if !output_dir.exists() {
        return Err(ChronicleError::Config(format!(
            "Output directory does not exist: {}",
            output_dir.display()
        )));
    }

    let mut chronicles = Vec::new();

    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(filename) = path.file_name() {
                if let Some(name) = filename.to_str() {
                    if name.starts_with("chronicle-") && name.ends_with(".md") {
                        chronicles.push(path);
                    }
                }
            }
        }
    }

    if chronicles.is_empty() {
        return Err(ChronicleError::Config(
            "No chronicle files found. Run 'chronicle gen' first.".to_string(),
        ));
    }

    // Sort by filename (which includes date)
    chronicles.sort();

    // Return the last one (most recent)
    Ok(chronicles.last().unwrap().clone())
}
