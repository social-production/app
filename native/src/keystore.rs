//! Keystore passphrase for this device's agent keys.

use std::path::Path;

use rand::RngCore;
use tauri_plugin_hc::{vec_to_locked, SharedLockedArray};

/// Random per-device passphrase, created on first run. M7: move to the OS keychain
/// (Keychain / Keystore / Secret Service) so the file can't be copied with the data.
pub fn passphrase(data_dir: &Path) -> std::io::Result<SharedLockedArray> {
    let path = data_dir.join("passphrase");
    if let Ok(existing) = std::fs::read(&path) {
        return Ok(vec_to_locked(existing));
    }
    let mut bytes = vec![0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    std::fs::write(&path, &bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(vec_to_locked(bytes))
}
