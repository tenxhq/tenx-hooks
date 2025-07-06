use rust_hook::utils::{find_project_root, is_rust_file};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_project_root() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested directory structure
    let nested_path = temp_dir.path().join("src").join("lib");
    fs::create_dir_all(&nested_path).unwrap();

    // Create Cargo.toml at root
    fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();

    // Create a file in the nested directory
    let file_path = nested_path.join("test.rs");
    fs::write(&file_path, "// test").unwrap();

    // Test finding project root
    let project_root = find_project_root(file_path.to_str().unwrap());
    assert_eq!(project_root, temp_dir.path().to_str().unwrap());
}

#[test]
fn test_rust_file_check() {
    assert!(is_rust_file("test.rs"));
    assert!(is_rust_file("/path/to/file.rs"));
    assert!(!is_rust_file("test.py"));
    assert!(!is_rust_file("test.txt"));
    assert!(!is_rust_file("test"));
}

#[test]
fn test_cli_help() {
    use std::process::Command;

    let output = Command::new("cargo")
        .args(["run", "--bin", "rust-hook", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rust-hook"));
    assert!(stdout.contains("posttool"));
    assert!(stdout.contains("stop"));
}

#[test]
fn test_subcommand_help() {
    use std::process::Command;

    // Test posttool help
    let output = Command::new("cargo")
        .args(["run", "--bin", "rust-hook", "--", "posttool", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("post-tool-use"));

    // Test stop help
    let output = Command::new("cargo")
        .args(["run", "--bin", "rust-hook", "--", "stop", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("stop"));
}
