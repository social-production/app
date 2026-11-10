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
        // Full arc while the network is small: every desktop node holds the whole directory, so
        // losing the seed node loses nothing (ADR 0036). Revisit once there are many full nodes.
        target_arc_factor: 1,
        advanced: Some(advanced),
        ..NetworkConfig::default()
    }
}

/// With `SP_NET_LOG` set, log this node's transport every 30 s (its relay URLs and peer
/// connections): CI's desktop smoke test reads it to tell "not connected" from "slow".
pub fn log_stats(handle: tauri::AppHandle) {
    use tauri_plugin_hc::HolochainExt;
    if std::env::var_os("SP_NET_LOG").is_none() {
        return;
    }
    tauri::async_runtime::spawn(async move {
        loop {
            if let Ok(runtime) = handle
                .holochain()
                .and_then(tauri_plugin_hc::HolochainPlugin::try_runtime)
            {
                match runtime.dump_network_stats().await {
                    Ok(stats) => {
                        let t = &stats.transport_stats;
                        log::warn!(
                            "network: backend {} urls {:?} connections {}",
                            t.backend,
                            t.peer_urls,
                            serde_json::to_string(&t.connections).unwrap_or_default()
                        );
                    }
                    Err(e) => log::warn!("network stats: {e}"),
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });
}
