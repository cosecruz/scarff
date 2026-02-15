//! Integration tests for scarff-cli.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scarff"))
        .stdout(predicate::str::contains("USAGE"));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_new_command_help() {
    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.args(&["new", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--lang"))
        .stdout(predicate::str::contains("--type"))
        .stdout(predicate::str::contains("--arch"));
}

#[test]
fn test_new_project_success() {
    let temp = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.current_dir(&temp);
    cmd.args(&[
        "new",
        "test-project",
        "--lang",
        "rust",
        "--type",
        "cli",
        "--arch",
        "layered",
        "--yes",
    ]);

    cmd.assert().success();

    // Verify project was created
    let project_path = temp.path().join("test-project");
    assert!(project_path.exists());
    assert!(project_path.join("src").exists());
    assert!(project_path.join("Cargo.toml").exists());
}

#[test]
fn test_new_project_dry_run() {
    let temp = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.current_dir(&temp);
    cmd.args(&[
        "new",
        "test-project",
        "--lang",
        "rust",
        "--type",
        "cli",
        "--arch",
        "layered",
        "--dry-run",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));

    // Verify project was NOT created
    let project_path = temp.path().join("test-project");
    assert!(!project_path.exists());
}

#[test]
fn test_new_project_already_exists() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path().join("existing-project");
    fs::create_dir(&project_path).unwrap();

    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.current_dir(&temp);
    cmd.args(&[
        "new",
        "existing-project",
        "--lang",
        "rust",
        "--type",
        "cli",
        "--arch",
        "layered",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_list_command() {
    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available Templates"));
}

#[test]
fn test_invalid_language() {
    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.args(&[
        "new", "test", "--lang", "java", "--type", "cli", "--arch", "layered",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported language"));
}

#[test]
fn test_verbose_flag() {
    let temp = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.current_dir(&temp);
    cmd.args(&[
        "-v",
        "new",
        "test-project",
        "--lang",
        "rust",
        "--type",
        "cli",
        "--arch",
        "layered",
        "--yes",
    ]);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("INFO")); // Should have log output
}

#[test]
fn test_quiet_flag() {
    let temp = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.current_dir(&temp);
    cmd.args(&[
        "-q",
        "new",
        "test-project",
        "--lang",
        "rust",
        "--type",
        "cli",
        "--arch",
        "layered",
        "--yes",
    ]);

    cmd.assert().success().stdout(predicate::str::is_empty()); // No output in quiet mode
}

#[test]
fn test_shell_completions() {
    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.args(&["completions", "bash"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}
