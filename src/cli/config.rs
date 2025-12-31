use crate::config::{self, Config};
use crate::error::Result;
use std::fs;
use std::path::PathBuf;

/// Initialize chronicle.toml configuration file
pub fn init(path: Option<PathBuf>) -> Result<()> {
    let config_path = path.unwrap_or_else(|| PathBuf::from("chronicle.toml"));

    // Check if file already exists
    if config_path.exists() {
        eprintln!(
            "Configuration file already exists at: {}",
            config_path.display()
        );
        eprintln!("Remove it first if you want to reinitialize.");
        return Ok(());
    }

    // Create default configuration
    let config = Config::default();

    // Create output directory if it doesn't exist
    if !config.output_dir.exists() {
        fs::create_dir_all(&config.output_dir)?;
        println!("Created output directory: {}", config.output_dir.display());
    }

    // Save configuration
    config::save(&config, &config_path)?;

    println!("Configuration file created: {}", config_path.display());
    println!("\nNext steps:");
    println!(
        "1. Edit {} to configure your repositories and files",
        config_path.display()
    );
    println!("2. Run 'chronicle gen' to generate your first chronicle");

    Ok(())
}
