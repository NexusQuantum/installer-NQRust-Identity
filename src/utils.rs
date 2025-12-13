use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::eyre::Result;

pub const ENV_TEMPLATE: &str = include_str!("../env_template");
pub const COMPOSE_TEMPLATE: &str = include_str!("../docker-compose.yaml");
pub const BOOTSTRAP_DOCKERFILE: &str = include_str!("../bootstrap/Dockerfile");
pub const BOOTSTRAP_INIT_SH: &str = include_str!("../bootstrap/init.sh");
pub const NORTHWIND_SQL: &str = include_str!("../northwind.sql");

pub fn find_file(filename: &str) -> bool {
    let root = project_root();
    root.join(filename).exists()
}

pub fn project_root() -> PathBuf {
    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Walk up to find a directory that contains either docker-compose files or Cargo.toml
    let candidates = [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
        "Cargo.toml",
    ];

    let mut current = start.as_path();
    while let Some(dir) = current.parent().or_else(|| Some(current)) {
        if candidates.iter().any(|name| dir.join(name).exists()) {
            return dir.to_path_buf();
        }

        // If we've reached filesystem root, stop
        if dir.parent().is_none() {
            break;
        }

        current = dir.parent().unwrap_or(dir);
    }

    // Fallback: if running from target/*/ build dirs, hop two parents as before
    if start
        .to_str()
        .map(|s| s.contains("target"))
        .unwrap_or(false)
    {
        if let Some(parent) = start.parent().and_then(|p| p.parent()) {
            return parent.to_path_buf();
        }
    }

    start
}

pub fn ensure_compose_bundle(root: &Path) -> Result<()> {
    // Compose file
    let compose_path = root.join("docker-compose.yaml");
    if !compose_path.exists() {
        if let Some(parent) = compose_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&compose_path, COMPOSE_TEMPLATE)?;
    }

    // Bootstrap build context
    let bootstrap_dir = root.join("bootstrap");
    fs::create_dir_all(&bootstrap_dir)?;

    let bootstrap_dockerfile = bootstrap_dir.join("Dockerfile");
    if !bootstrap_dockerfile.exists() {
        fs::write(&bootstrap_dockerfile, BOOTSTRAP_DOCKERFILE)?;
    }

    let bootstrap_init = bootstrap_dir.join("init.sh");
    if !bootstrap_init.exists() {
        fs::write(&bootstrap_init, BOOTSTRAP_INIT_SH)?;
        // Best-effort exec bit on Unix; ignore on other platforms.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o755);
            let _ = fs::set_permissions(&bootstrap_init, perms);
        }
    }

    // Northwind demo data
    let northwind_path = root.join("northwind.sql");
    if !northwind_path.exists() {
        fs::write(&northwind_path, NORTHWIND_SQL)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_file_exists() {
        // Cargo.toml is known to exist in the project root
        assert!(
            find_file("Cargo.toml"),
            "Should find Cargo.toml in project root"
        );
    }

    #[test]
    fn test_find_file_not_exists() {
        // This file should not exist
        assert!(
            !find_file("non_existent_file_xyz"),
            "Should not find non-existent file"
        );
    }
}
