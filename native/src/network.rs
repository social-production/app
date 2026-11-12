//! The conductor's network settings (docs/NETWORK.md); which server: `servers.rs`.

use tauri_plugin_hc::NetworkConfig;

use crate::servers::Server;

/// The conductor's network for `server` (picked by `servers::choose`, ADR 0091).
pub fn config(server: &Server) -> NetworkConfig {
    let mut advanced: serde_json::Value =
        serde_json::from_str(sp_core::network::GOSSIP_TUNING_JSON).unwrap_or_default();
    if server.relay.starts_with("http://") {
        // Local dev, and desktops that help the network on a forwarded port (ADR 0094).
        // iroh's traffic through a relay is end-to-end encrypted either way.
        advanced["irohTransport"] = serde_json::json!({ "relayAllowPlainText": true });
    }
    NetworkConfig {
        bootstrap_url: url2::Url2::parse(&server.bootstrap),
        relay_url: url2::Url2::parse(&server.relay),
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
