use assert_cmd::cargo;

#[test]
fn test_gen_dry_run_with_no_color() {
    std::env::set_var("NO_COLOR", "1");

    let mut cmd = cargo::cargo_bin_cmd!("chronicle");
    cmd.arg("gen").arg("--dry-run").assert().success();

    std::env::remove_var("NO_COLOR");
}

#[test]
fn test_gen_dry_run_with_clicolor_force() {
    std::env::set_var("CLICOLOR_FORCE", "1");

    let mut cmd = cargo::cargo_bin_cmd!("chronicle");
    cmd.arg("gen").arg("--dry-run").assert().success();

    std::env::remove_var("CLICOLOR_FORCE");
}
