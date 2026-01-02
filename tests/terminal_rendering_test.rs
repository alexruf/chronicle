use assert_cmd::Command;

#[test]
fn test_gen_dry_run_with_no_color() {
    std::env::set_var("NO_COLOR", "1");

    let mut cmd = Command::cargo_bin("chronicle").unwrap();
    cmd.arg("gen").arg("--dry-run").assert().success();

    std::env::remove_var("NO_COLOR");
}

#[test]
fn test_gen_dry_run_with_clicolor_force() {
    std::env::set_var("CLICOLOR_FORCE", "1");

    let mut cmd = Command::cargo_bin("chronicle").unwrap();
    cmd.arg("gen").arg("--dry-run").assert().success();

    std::env::remove_var("CLICOLOR_FORCE");
}
