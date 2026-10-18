//! Which bootstrap/relay this device uses (docs/NETWORK.md).

use tauri_plugin_hc::NetworkConfig;

/// The Railway bootstrap + relay (node #1). `SP_BOOTSTRAP_URL` overrides it, e.g. a
/// local `http://127.0.0.1:4433` from scripts/dev-seed.sh. M1+: a list, with failover.
const DEFAULT_BOOTSTRAP: &str = "https://bootstrap-production-f978.up.railway.app";

pub fn config() -> NetworkConfig {
    let url = std::env::var("SP_BOOTSTRAP_URL").unwrap_or_else(|_| DEFAULT_BOOTSTRAP.into());
    let mut advanced: serde_json::Value =
        serde_json::from_str(sp_core::network::GOSSIP_TUNING_JSON).unwrap_or_default();
    if url.starts_with("http://") {
        // Local dev only: production relays are always https.
        advanced["irohTransport"] = serde_json::json!({ "relayAllowPlainText": true });
    }
    NetworkConfig {
        bootstrap_url: url2::Url2::parse(&url),
        relay_url: url2::Url2::parse(&url),
        advanced: Some(advanced),
        ..NetworkConfig::default()
    }
}
