use assert_cmd::cargo;
use serial_test::serial;
use tempfile::TempDir;

#[test]
#[serial]
fn test_gen_dry_run_with_no_color() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("chronicle.toml");

    // Create config
    cargo::cargo_bin_cmd!("chronicle")
        .args(["config", "init", "--path", config_path.to_str().unwrap()])
        .assert()
        .success();

    std::env::set_var("NO_COLOR", "1");

    let mut cmd = cargo::cargo_bin_cmd!("chronicle");
    cmd.arg("gen")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("--dry-run")
        .assert()
        .success();

    std::env::remove_var("NO_COLOR");
}

#[test]
#[serial]
fn test_gen_dry_run_with_clicolor_force() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("chronicle.toml");

    // Create config
    cargo::cargo_bin_cmd!("chronicle")
        .args(["config", "init", "--path", config_path.to_str().unwrap()])
        .assert()
        .success();

    std::env::set_var("CLICOLOR_FORCE", "1");

    let mut cmd = cargo::cargo_bin_cmd!("chronicle");
    cmd.arg("gen")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("--dry-run")
        .assert()
        .success();

    std::env::remove_var("CLICOLOR_FORCE");
}
