//! Tests for error handling and suggestions.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_error_with_suggestions_unsupported_language() {
    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.args(&[
        "new", "test", "--lang", "go", "--type", "cli", "--arch", "layered",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported language"))
        .stderr(predicate::str::contains("rust"))
        .stderr(predicate::str::contains("python"))
        .stderr(predicate::str::contains("typescript"));
}

#[test]
fn test_error_with_suggestions_framework_mismatch() {
    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.args(&[
        "new",
        "test",
        "--lang",
        "rust",
        "--type",
        "cli",
        "--arch",
        "layered",
        "--framework",
        "django", // Wrong framework for Rust
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not available"))
        .stderr(predicate::str::contains("axum"))
        .stderr(predicate::str::contains("actix"));
}

#[test]
fn test_error_invalid_project_name() {
    let mut cmd = Command::cargo_bin("scarff").unwrap();
    cmd.args(&[
        "new", ".hidden", "--lang", "rust", "--type", "cli", "--arch", "layered",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid project name"));
}
