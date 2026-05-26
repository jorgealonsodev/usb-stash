//! Integration tests for the `usbstash` CLI binary.
//!
//! Tests the full round-trip: create → add → list → extract.

use std::fs;

use assert_cmd::Command as AssertCommand;
use predicates::prelude::*;
use tempfile::tempdir;

fn usbstash() -> AssertCommand {
    AssertCommand::cargo_bin("usbstash").unwrap()
}

#[test]
fn round_trip_create_add_list_extract() {
    let dir = tempdir().unwrap();
    let stash_path = dir.path();
    let password = "test-password";

    // 1. Create
    usbstash()
        .arg("create")
        .arg(stash_path)
        .arg("--password")
        .arg(password)
        .assert()
        .success()
        .stderr(predicate::str::contains("Stash created"));

    // Verify stash files exist
    assert!(stash_path.join("stash.meta").exists());
    assert!(stash_path.join("stash.dat").exists());

    // 2. Add a file
    let src_file = dir.path().join("notes.txt");
    fs::write(&src_file, "hello world from integration test").unwrap();

    usbstash()
        .arg("add")
        .arg(stash_path)
        .arg(&src_file)
        .arg("--password")
        .arg(password)
        .assert()
        .success()
        .stderr(predicate::str::contains("Added"));

    // 3. List entries
    let output = usbstash()
        .arg("list")
        .arg(stash_path)
        .arg("--password")
        .arg(password)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("notes.txt"));
    assert!(stdout.contains("text/plain"));

    // 4. Extract
    let extracted = dir.path().join("extracted.txt");
    usbstash()
        .arg("extract")
        .arg(stash_path)
        .arg("notes.txt")
        .arg("--output")
        .arg(&extracted)
        .arg("--password")
        .arg(password)
        .assert()
        .success()
        .stderr(predicate::str::contains("Extracted"));

    // Verify extracted content matches original
    let content = fs::read_to_string(&extracted).unwrap();
    assert_eq!(content, "hello world from integration test");
}

#[test]
fn create_already_exists_exit_code_1() {
    let dir = tempdir().unwrap();
    let password = "test-password";

    // Create once
    usbstash()
        .arg("create")
        .arg(dir.path())
        .arg("--password")
        .arg(password)
        .assert()
        .success();

    // Try to create again — should fail with exit code 1
    usbstash()
        .arg("create")
        .arg(dir.path())
        .arg("--password")
        .arg(password)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn add_file_not_found_exit_code_1() {
    let dir = tempdir().unwrap();
    let password = "test-password";

    // Create stash
    usbstash()
        .arg("create")
        .arg(dir.path())
        .arg("--password")
        .arg(password)
        .assert()
        .success();

    // Try to add non-existent file
    usbstash()
        .arg("add")
        .arg(dir.path())
        .arg("/nonexistent/file.txt")
        .arg("--password")
        .arg(password)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("file not found"));
}

#[test]
fn list_empty_stash_exit_code_0() {
    let dir = tempdir().unwrap();
    let password = "test-password";

    usbstash()
        .arg("create")
        .arg(dir.path())
        .arg("--password")
        .arg(password)
        .assert()
        .success();

    usbstash()
        .arg("list")
        .arg(dir.path())
        .arg("--password")
        .arg(password)
        .assert()
        .success()
        .stdout(predicate::str::contains("No entries."));
}

#[test]
fn list_wrong_password_exit_code_1() {
    let dir = tempdir().unwrap();
    let password = "correct";

    usbstash()
        .arg("create")
        .arg(dir.path())
        .arg("--password")
        .arg(password)
        .assert()
        .success();

    usbstash()
        .arg("list")
        .arg(dir.path())
        .arg("--password")
        .arg("wrong")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn extract_entry_not_found_exit_code_1() {
    let dir = tempdir().unwrap();
    let password = "test-password";

    usbstash()
        .arg("create")
        .arg(dir.path())
        .arg("--password")
        .arg(password)
        .assert()
        .success();

    usbstash()
        .arg("extract")
        .arg(dir.path())
        .arg("missing.txt")
        .arg("--password")
        .arg(password)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn extract_output_already_exists_exit_code_1() {
    let dir = tempdir().unwrap();
    let password = "test-password";

    usbstash()
        .arg("create")
        .arg(dir.path())
        .arg("--password")
        .arg(password)
        .assert()
        .success();

    // Add a file
    let src = dir.path().join("notes.txt");
    fs::write(&src, "hello").unwrap();
    usbstash()
        .arg("add")
        .arg(dir.path())
        .arg(&src)
        .arg("--password")
        .arg(password)
        .assert()
        .success();

    // Pre-create output file
    let output = dir.path().join("existing.txt");
    fs::write(&output, "existing").unwrap();

    usbstash()
        .arg("extract")
        .arg(dir.path())
        .arg("notes.txt")
        .arg("--output")
        .arg(&output)
        .arg("--password")
        .arg(password)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("file already exists"));
}

#[test]
fn clap_usage_error_exit_code_2() {
    // Invalid subcommand should exit with code 2 (clap default)
    usbstash().arg("invalid-command").assert().failure().code(2);
}

#[test]
fn add_with_as_path_override() {
    let dir = tempdir().unwrap();
    let password = "test-password";

    usbstash()
        .arg("create")
        .arg(dir.path())
        .arg("--password")
        .arg(password)
        .assert()
        .success();

    let src = dir.path().join("long-path-file.txt");
    fs::write(&src, "content").unwrap();

    usbstash()
        .arg("add")
        .arg(dir.path())
        .arg(&src)
        .arg("--as")
        .arg("short.txt")
        .arg("--password")
        .arg(password)
        .assert()
        .success();

    // List should show "short.txt" not "long-path-file.txt"
    let output = usbstash()
        .arg("list")
        .arg(dir.path())
        .arg("--password")
        .arg(password)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("short.txt"));
    assert!(!stdout.contains("long-path-file.txt"));
}
