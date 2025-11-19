use predicates::prelude::*;

#[test]
fn test_help_flag() {
    // Verify that `chain-ping --help` runs and prints usage info
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("chain-ping");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_version_flag() {
    // Verify that `chain-ping --version` works
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("chain-ping");
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("chain-ping"));
}

#[test]
fn test_missing_args_fails() {
    // Verify that running without arguments returns an error
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("chain-ping");
    cmd.assert()
        .failure() // Should fail
        .stderr(predicate::str::contains("Error: At least one endpoint"));
}