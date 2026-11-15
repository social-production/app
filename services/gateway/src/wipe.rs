//! A one-time fresh start for the seed node's data (owner, 2026-11-16: test data only).
//!
//! After the project reset of 2026-11-15 (PR #160) the live keystore failed every request
//! ("lair connection recv loop ended", `ConnectionAborted`), so the archivist could not be
//! installed and the node hosted nothing. Its storage (the keystore, the conductor's
//! databases and `gateway.sqlite`, all on the `/data` volume) is cleared once: when the
//! `generation` file in the data directory does not hold `GENERATION`, everything in it is
//! removed and the file is written. Bump `GENERATION` to wipe again; leave it to keep data.
use std::path::Path;

use anyhow::Context;

const GENERATION: &str = "2026-11-16";

/// Clear `data_dir` unless it already holds this generation's data.
pub fn fresh_start_once(data_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(data_dir)?;
    let marker = data_dir.join("generation");
    let current = std::fs::read_to_string(&marker).unwrap_or_default();
    if current.trim() == GENERATION {
        return Ok(());
    }
    let before = free_mb(data_dir);
    let mut removed = 0;
    for entry in std::fs::read_dir(data_dir)? {
        let path = entry?.path();
        if path.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        }
        .with_context(|| format!("remove {}", path.display()))?;
        removed += 1;
    }
    std::fs::write(&marker, GENERATION)?;
    tracing::warn!(
        generation = GENERATION,
        removed,
        free_mb_before = before,
        free_mb_after = free_mb(data_dir),
        "data directory cleared for a fresh start"
    );
    Ok(())
}

/// Free space on the data directory's disk in MB, for the log (0 if unknown).
fn free_mb(dir: &Path) -> u64 {
    nix::sys::statvfs::statvfs(dir)
        .map_or(0, |s| s.blocks_available() * s.fragment_size() / 1_048_576)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clears_once_then_keeps() -> anyhow::Result<()> {
        let dir = std::env::temp_dir().join(format!("wipe-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("conductor"))?;
        std::fs::write(dir.join("gateway.sqlite"), b"old")?;
        fresh_start_once(&dir)?;
        assert!(!dir.join("conductor").exists());
        assert!(!dir.join("gateway.sqlite").exists());
        std::fs::write(dir.join("gateway.sqlite"), b"new")?;
        fresh_start_once(&dir)?;
        assert!(dir.join("gateway.sqlite").exists());
        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
