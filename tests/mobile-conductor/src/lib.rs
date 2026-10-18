//! Compile-only check that the embedded conductor builds for iOS/Android
//! (`.github/workflows/platforms.yml`). The real mobile entry point lands in M7 in `native/`.

use holochain::conductor::config::ConductorConfig;
use holochain::conductor::{Conductor, ConductorHandle};

/// Same call the native app makes; referenced so it is linked for the target.
pub async fn start(config: ConductorConfig) -> anyhow::Result<ConductorHandle> {
    Conductor::builder()
        .config(config)
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("{e:?}"))
}
