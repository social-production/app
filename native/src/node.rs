//! This device's node: an in-process conductor plus the gateway on localhost.
//! Data (chain, DHT shards, keystore) lives in the OS app-data dir.

use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Context;
use holochain::conductor::config::{
    AdminInterfaceConfig, ConductorConfig, InterfaceDriver, KeystoreConfig, NetworkConfig,
};
use holochain::conductor::{Conductor, ConductorHandle};
use holochain_types::websocket::AllowedOrigins;
use rand::RngCore;

/// Bootstrap/relay for this device. Defaults to Holochain's public dev-test servers
/// until the Railway bootstrap service exists; set `SP_BOOTSTRAP_URL` to override.
const BOOTSTRAP_ENV: &str = "SP_BOOTSTRAP_URL";

/// A running node: keep the handle alive for the life of the app.
pub struct Node {
    pub conductor: ConductorHandle,
    /// The local gateway, `http://127.0.0.1:<port>`.
    pub gateway_url: String,
}

/// Start the conductor and the gateway in front of it.
pub async fn start(data_dir: PathBuf) -> anyhow::Result<Node> {
    std::fs::create_dir_all(&data_dir)?;
    let admin_port = free_port()?;
    let app_ws_port = free_port()?;

    let passphrase = load_or_create_passphrase(&data_dir.join("passphrase"))?;
    let config = conductor_config(&data_dir, admin_port);
    let conductor = Conductor::builder()
        .config(config)
        .passphrase(Some(Arc::new(Mutex::new(passphrase.into()))))
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("start conductor: {e:?}"))?;
    let gateway_config = gateway::Config {
        bind_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        admin_port,
        app_ws_port,
        happ_path: data_dir.join("social_production.happ"),
    };
    std::fs::write(&gateway_config.happ_path, HAPP).context("write bundled hApp")?;

    let (bound_tx, bound_rx) = tokio::sync::oneshot::channel();
    tauri::async_runtime::spawn(async move {
        let on_bound = move |addr: SocketAddr| {
            let _ = bound_tx.send(addr);
        };
        if let Err(e) = gateway::serve(gateway_config, on_bound, std::future::pending()).await {
            tracing::error!(error = %e, "local gateway stopped");
        }
    });
    let addr = bound_rx.await.context("local gateway failed to start")?;
    Ok(Node {
        conductor,
        gateway_url: format!("http://{addr}"),
    })
}

/// The hApp is compiled into the binary so the app needs no separate download.
const HAPP: &[u8] = include_bytes!("../../happ/social_production.happ");

fn conductor_config(data_dir: &Path, admin_port: u16) -> ConductorConfig {
    let mut network = NetworkConfig::default();
    if let Ok(url) = std::env::var(BOOTSTRAP_ENV) {
        network.bootstrap_url = url2::Url2::parse(&url);
        network.relay_url = url2::Url2::parse(&url);
        if url.starts_with("http://") {
            // Local dev only: production relays are always https.
            network.advanced = Some(serde_json::json!({
                "irohTransport": { "relayAllowPlainText": true }
            }));
        }
    }
    ConductorConfig {
        data_root_path: Some(data_dir.join("conductor").into()),
        keystore: KeystoreConfig::LairServerInProc { lair_root: None },
        admin_interfaces: Some(vec![AdminInterfaceConfig {
            driver: InterfaceDriver::Websocket {
                port: admin_port,
                danger_bind_addr: None,
                allowed_origins: AllowedOrigins::Any,
            },
        }]),
        network,
        ..Default::default()
    }
}

/// Random per-device keystore passphrase. M7: move to the OS keychain.
fn load_or_create_passphrase(path: &Path) -> anyhow::Result<Vec<u8>> {
    if let Ok(existing) = std::fs::read(path) {
        return Ok(existing);
    }
    let mut bytes = vec![0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    std::fs::write(path, &bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(bytes)
}

fn free_port() -> anyhow::Result<u16> {
    Ok(TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?
        .local_addr()?
        .port())
}
