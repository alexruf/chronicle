use assert_cmd::cargo;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// Helper to convert path to forward slashes for TOML compatibility on Windows
fn path_to_toml_string(path: &std::path::Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// Helper to create a test Git repository with commits
fn create_test_git_repo(path: &std::path::Path) {
    // Initialize Git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .unwrap();

    // Configure Git
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()
        .unwrap();

    // Create initial commit
    fs::write(path.join("test.txt"), "initial content").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()
        .unwrap();

    // Create another commit
    fs::write(path.join("test.txt"), "updated content").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Update file"])
        .current_dir(path)
        .output()
        .unwrap();
}

#[test]
fn test_config_init() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("chronicle.toml");

    cargo::cargo_bin_cmd!("chronicle")
        .args(["config", "init", "--path", config_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration file created"));

    assert!(config_path.exists());
}

#[test]
fn test_state_reset() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("chronicle.toml");
    let state_file = temp_dir.path().join(".chronicle-state.json");

    // Create config
    cargo::cargo_bin_cmd!("chronicle")
        .args(["config", "init", "--path", config_path.to_str().unwrap()])
        .assert()
        .success();

    // Update config to set correct state_file path
    let config_content = fs::read_to_string(&config_path).unwrap();
    let updated_config = config_content.replace(
        "state_file = \"./.chronicle-state.json\"",
        &format!("state_file = \"{}\"", path_to_toml_string(&state_file)),
    );
    fs::write(&config_path, updated_config).unwrap();

    // Create dummy state file
    fs::write(
        &state_file,
        r#"{"version":"1.0","last_updated":"2024-01-01T00:00:00Z","sources":{}}"#,
    )
    .unwrap();

    // Reset state
    cargo::cargo_bin_cmd!("chronicle")
        .args(["state", "reset", "--config", config_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("State file deleted"));

    assert!(!state_file.exists());
}

#[test]
fn test_gen_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo");
    fs::create_dir(&repo_path).unwrap();
    create_test_git_repo(&repo_path);

    let config_path = temp_dir.path().join("chronicle.toml");

    // Create config
    cargo::cargo_bin_cmd!("chronicle")
        .args(["config", "init", "--path", config_path.to_str().unwrap()])
        .assert()
        .success();

    // Update config to include the test repo
    let config_content = fs::read_to_string(&config_path).unwrap();
    let updated_config = config_content.replace(
        "repos = [\".\"]",
        &format!("repos = [\"{}\"]", path_to_toml_string(&repo_path)),
    );
    fs::write(&config_path, updated_config).unwrap();

    // Run gen with dry-run
    cargo::cargo_bin_cmd!("chronicle")
        .args([
            "gen",
            "--config",
            config_path.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Chronicle:"))
        .stdout(predicate::str::contains("## Summary"))
        .stdout(predicate::str::contains("## Git Activity"));
}

#[test]
fn test_gen_and_show_latest() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo");
    fs::create_dir(&repo_path).unwrap();
    create_test_git_repo(&repo_path);

    let config_path = temp_dir.path().join("chronicle.toml");
    let chronicles_dir = temp_dir.path().join("chronicles");

    // Create config
    cargo::cargo_bin_cmd!("chronicle")
        .args(["config", "init", "--path", config_path.to_str().unwrap()])
        .assert()
        .success();

    // Update config to set output_dir and add repo
    let config_content = fs::read_to_string(&config_path).unwrap();
    let updated_config = config_content
        .replace(
            "output_dir = \"./chronicles\"",
            &format!("output_dir = \"{}\"", path_to_toml_string(&chronicles_dir)),
        )
        .replace(
            "repos = [\".\"]",
            &format!("repos = [\"{}\"]", path_to_toml_string(&repo_path)),
        );
    fs::write(&config_path, updated_config).unwrap();

    // Run gen
    cargo::cargo_bin_cmd!("chronicle")
        .args(["gen", "--config", config_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Chronicle written to"));

    // Verify chronicle file was created
    assert!(chronicles_dir.exists());
    let files: Vec<_> = fs::read_dir(&chronicles_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);

    // Run show latest
    cargo::cargo_bin_cmd!("chronicle")
        .args(["show", "latest", "--config", config_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Chronicle:"))
        .stdout(predicate::str::contains("Update file"));
}

#[test]
fn test_gen_with_todos() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("chronicle.toml");
    let todo_file = temp_dir.path().join("todo.md");
    let chronicles_dir = temp_dir.path().join("chronicles");

    // Create TODO file
    fs::write(
        &todo_file,
        r#"# My TODOs
- [ ] Task 1
- [x] Task 2
- [~] Task 3
"#,
    )
    .unwrap();

    // Create config
    cargo::cargo_bin_cmd!("chronicle")
        .args(["config", "init", "--path", config_path.to_str().unwrap()])
        .assert()
        .success();

    // Update config
    let config_content = fs::read_to_string(&config_path).unwrap();
    let updated_config = config_content
        .replace(
            "output_dir = \"./chronicles\"",
            &format!("output_dir = \"{}\"", path_to_toml_string(&chronicles_dir)),
        )
        .replace(
            "todo_files = []",
            &format!("todo_files = [\"{}\"]", path_to_toml_string(&todo_file)),
        );
    fs::write(&config_path, updated_config).unwrap();

    // Run gen
    cargo::cargo_bin_cmd!("chronicle")
        .args(["gen", "--config", config_path.to_str().unwrap()])
        .assert()
        .success();

    // Verify TODOs are in output
    cargo::cargo_bin_cmd!("chronicle")
        .args(["show", "latest", "--config", config_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("## TODOs"))
        .stdout(predicate::str::contains("Task 1"))
        .stdout(predicate::str::contains("Task 2"))
        .stdout(predicate::str::contains("Task 3"));
}

#[test]
fn test_incremental_updates() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo");
    fs::create_dir(&repo_path).unwrap();
    create_test_git_repo(&repo_path);

    let config_path = temp_dir.path().join("chronicle.toml");
    let chronicles_dir = temp_dir.path().join("chronicles");

    // Create config
    cargo::cargo_bin_cmd!("chronicle")
        .args(["config", "init", "--path", config_path.to_str().unwrap()])
        .assert()
        .success();

    // Update config
    let config_content = fs::read_to_string(&config_path).unwrap();
    let updated_config = config_content
        .replace(
            "output_dir = \"./chronicles\"",
            &format!("output_dir = \"{}\"", path_to_toml_string(&chronicles_dir)),
        )
        .replace(
            "repos = [\".\"]",
            &format!("repos = [\"{}\"]", path_to_toml_string(&repo_path)),
        );
    fs::write(&config_path, updated_config).unwrap();

    // First gen
    cargo::cargo_bin_cmd!("chronicle")
        .args(["gen", "--config", config_path.to_str().unwrap()])
        .assert()
        .success();

    // Create another commit
    fs::write(repo_path.join("new.txt"), "new file").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-m", "Add new file"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Second gen with new date
    let tomorrow = chrono::Local::now().date_naive() + chrono::Duration::days(1);
    cargo::cargo_bin_cmd!("chronicle")
        .args([
            "gen",
            "--config",
            config_path.to_str().unwrap(),
            "--date",
            &tomorrow.format("%Y-%m-%d").to_string(),
        ])
        .assert()
        .success();

    // Verify we have two chronicle files
    let files_count = fs::read_dir(&chronicles_dir).unwrap().count();
    assert_eq!(files_count, 2);
}
