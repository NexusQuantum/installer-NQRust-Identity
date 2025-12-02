use std::path::PathBuf;

pub const ENV_TEMPLATE: &str = include_str!("../env_template");

pub fn find_file(filename: &str) -> bool {
    let root = project_root();
    root.join(filename).exists()
}

pub fn project_root() -> PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if current
        .to_str()
        .map(|s| s.contains("target"))
        .unwrap_or(false)
    {
        if let Some(parent) = current.parent().and_then(|p| p.parent()) {
            current = parent.to_path_buf();
        }
    }

    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_file_exists() {
        // Cargo.toml is known to exist in the project root
        assert!(find_file("Cargo.toml"), "Should find Cargo.toml in project root");
    }

    #[test]
    fn test_find_file_not_exists() {
        // This file should not exist
        assert!(!find_file("non_existent_file_xyz"), "Should not find non-existent file");
    }
}
