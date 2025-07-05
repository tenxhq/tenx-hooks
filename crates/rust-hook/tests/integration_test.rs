use std::fs;
use std::path::Path;
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

// Helper functions copied from main.rs for testing
fn find_project_root(file_path: &str) -> String {
    let path = Path::new(file_path);
    let mut current = path.parent();

    while let Some(dir) = current {
        if dir.join("Cargo.toml").exists() {
            return dir.to_string_lossy().to_string();
        }
        current = dir.parent();
    }

    path.parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string())
}

fn is_rust_file(file_path: &str) -> bool {
    file_path.ends_with(".rs")
}
