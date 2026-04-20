use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::protocol::SessionRegistryEntry;

pub fn registry_root() -> Result<PathBuf> {
    let root = std::env::current_dir()?.join(".tuiless");
    std::fs::create_dir_all(&root)?;
    Ok(root)
}

pub fn registry_file(session_key: &str) -> Result<PathBuf> {
    Ok(registry_root()?.join(format!("{session_key}.json")))
}

pub fn write_entry(entry: &SessionRegistryEntry) -> Result<()> {
    let path = registry_file(&entry.session_key)?;
    let content = serde_json::to_vec_pretty(entry)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn read_entry(session_key: &str) -> Result<Option<SessionRegistryEntry>> {
    let path = registry_file(session_key)?;
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(&path)
        .with_context(|| format!("failed to read session registry {}", path.display()))?;
    let entry = serde_json::from_slice::<SessionRegistryEntry>(&bytes)
        .with_context(|| format!("failed to decode session registry {}", path.display()))?;
    Ok(Some(entry))
}

pub fn delete_entry(session_key: &str) -> Result<()> {
    let path = registry_file(session_key)?;
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn registry_file_for_path(session_key: &str) -> Result<PathBuf> {
    registry_file(session_key)
}

#[allow(dead_code)]
pub fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
