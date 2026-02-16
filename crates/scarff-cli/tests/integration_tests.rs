// //! Integration tests for scarff-cli.

// use assert_cmd::cargo;
// use assert_cmd::prelude::*;
// use predicates::prelude::*;
// use std::fs;
// use tempfile::TempDir;

// #[test]
// fn test_help_flag() {
//     let mut cmd = cargo::cargo_bin_cmd!("scarff");
//     cmd.arg("--help")
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("Scarff"))
//         .stdout(predicate::str::contains("USAGE"));
// }

// #[test]
// fn test_version_flag() {
//     let mut cmd = cargo::cargo_bin_cmd!("scarff");
//     cmd.arg("--version")
//         .assert()
//         .success()
//         .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
// }

// #[test]
// fn test_new_command_help() {
//     let mut cmd = cargo::cargo_bin_cmd!("scarff");
//     cmd.args(["new", "--help"])
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("--lang"))
//         .stdout(predicate::str::contains("--type"))
//         .stdout(predicate::str::contains("--arch"));
// }

// #[test]
// fn test_new_project_success() {
//     let temp = TempDir::new().unwrap();
//     let mut cmd = cargo::cargo_bin_cmd!("scarff");

//     cmd.current_dir(temp.path())
//         .args([
//             "new",
//             "test-project",
//             "--lang",
//             "rust",
//             "--type",
//             "cli",
//             "--arch",
//             "layered",
//             "--yes",
//         ])
//         .assert()
//         .success();

//     let project_path = temp.path().join("test-project");
//     assert!(project_path.exists());
//     assert!(project_path.join("src").exists());
//     assert!(project_path.join("Cargo.toml").exists());
// }

// #[test]
// fn test_new_project_dry_run() {
//     let temp = TempDir::new().unwrap();
//     let mut cmd = cargo::cargo_bin_cmd!("scarff");

//     cmd.current_dir(temp.path())
//         .args([
//             "new",
//             "test-project",
//             "--lang",
//             "rust",
//             "--type",
//             "cli",
//             "--arch",
//             "layered",
//             "--dry-run",
//         ])
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("Dry run"));

//     assert!(!temp.path().join("test-project").exists());
// }

// #[test]
// fn test_new_project_already_exists() {
//     let temp = TempDir::new().unwrap();
//     fs::create_dir(temp.path().join("existing-project")).unwrap();

//     let mut cmd = cargo::cargo_bin_cmd!("scarff");
//     cmd.current_dir(temp.path())
//         .args([
//             "new",
//             "existing-project",
//             "--lang",
//             "rust",
//             "--type",
//             "cli",
//             "--arch",
//             "layered",
//         ])
//         .assert()
//         .failure()
//         .stderr(predicate::str::contains("already exists"));
// }

// #[test]
// fn test_list_command() {
//     let mut cmd = cargo::cargo_bin_cmd!("scarff");
//     cmd.arg("list")
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("Available Templates"));
// }

// #[test]
// fn test_invalid_language() {
//     let mut cmd = cargo::cargo_bin_cmd!("scarff");
//     cmd.args([
//         "new", "test", "--lang", "java", "--type", "cli", "--arch", "layered",
//     ])
//     .assert()
//     .failure()
//     .stderr(predicate::str::contains("Unsupported language"));
// }

// #[test]
// fn test_verbose_flag() {
//     let temp = TempDir::new().unwrap();
//     let mut cmd = cargo::cargo_bin_cmd!("scarff");

//     cmd.current_dir(temp.path())
//         .args([
//             "-v",
//             "new",
//             "test-project",
//             "--lang",
//             "rust",
//             "--type",
//             "cli",
//             "--arch",
//             "layered",
//             "--yes",
//         ])
//         .assert()
//         .success()
//         .stderr(predicate::str::contains("INFO"));
// }

// #[test]
// fn test_quiet_flag() {
//     let temp = TempDir::new().unwrap();
//     let mut cmd = cargo::cargo_bin_cmd!("scarff");

//     cmd.current_dir(temp.path())
//         .args([
//             "-q",
//             "new",
//             "test-project",
//             "--lang",
//             "rust",
//             "--type",
//             "cli",
//             "--arch",
//             "layered",
//             "--yes",
//         ])
//         .assert()
//         .success()
//         .stdout(predicate::str::is_empty());
// }

// #[test]
// fn test_shell_completions() {
//     let mut cmd = cargo::cargo_bin_cmd!("scarff");
//     cmd.args(["completions", "bash"])
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("complete"));
// }
