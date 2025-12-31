use crate::config;
use crate::error::Result;
use std::fs;
use std::path::PathBuf;

/// Reset state tracking by deleting the state file
pub fn reset(config_path: Option<PathBuf>) -> Result<()> {
    let config_path = config_path.unwrap_or_else(|| PathBuf::from("chronicle.toml"));

    // Load config to get state file path
    let config = config::load(&config_path)?;

    // Delete state file if it exists
    if config.state_file.exists() {
        fs::remove_file(&config.state_file)?;
        println!("State file deleted: {}", config.state_file.display());
        println!("Next 'chronicle gen' will generate a full chronicle.");
    } else {
        println!("State file does not exist: {}", config.state_file.display());
        println!("Nothing to reset.");
    }

    Ok(())
}
