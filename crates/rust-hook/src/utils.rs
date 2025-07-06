use std::path::Path;

/// Find the nearest ancestor directory containing a `Cargo.toml` file.
/// Returns the directory path as a `String`.
pub fn find_project_root(file_path: &str) -> String {
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

/// Determine whether the provided path refers to a Rust source file.
pub fn is_rust_file(file_path: &str) -> bool {
    file_path.ends_with(".rs")
}
