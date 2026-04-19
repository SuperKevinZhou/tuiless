use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

pub fn canonical_session_key(cwd: &Path) -> Result<String> {
    let canonical = cwd
        .canonicalize()
        .or_else(|_| normalize_fallback(cwd))
        .context("failed to normalize cwd for session key")?;
    let normalized = canonical.to_string_lossy().to_ascii_lowercase();
    let digest = Sha256::digest(normalized.as_bytes());
    Ok(hex::encode(digest))
}

pub fn normalize_cwd(cwd: &Path) -> Result<PathBuf> {
    cwd.canonicalize()
        .or_else(|_| normalize_fallback(cwd))
        .context("failed to canonicalize cwd")
}

fn normalize_fallback(path: &Path) -> Result<PathBuf> {
    let base = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::canonical_session_key;

    #[test]
    fn equivalent_cwd_paths_produce_same_session_key() {
        let base = std::env::current_dir().unwrap();
        let path_a = base.join(".");
        assert_eq!(
            canonical_session_key(&path_a).unwrap(),
            canonical_session_key(&base).unwrap()
        );
    }
}
