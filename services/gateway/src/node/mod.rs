//! The seed node's conductor, embedded in this process. There is no admin websocket:
//! admin calls go straight to the conductor (as Holochain's runtime-tauri does).

mod hosting;
mod seeds;

use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use holochain::conductor::api::{AdminInterfaceApi, AdminRequest, AdminResponse};
use holochain::conductor::config::{ConductorConfig, KeystoreConfig, NetworkConfig};
use holochain::conductor::{ConductorBuilder, ConductorHandle};
use holochain_keystore::MetaLairClient;
use holochain_types::websocket::AllowedOrigins;

pub use hosting::app_id;

use crate::config::Config;

#[derive(Clone)]
pub struct Node {
    conductor: ConductorHandle,
    keystore: MetaLairClient,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Node")
    }
}

impl Node {
    /// Open the keystore, boot the conductor and expose the app interface on localhost.
    pub async fn start(config: &Config, passphrase: Vec<u8>) -> anyhow::Result<Self> {
        // Network TLS needs a process-wide crypto provider; a second install is harmless.
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let data_root = config.data_dir.join("conductor");
        std::fs::create_dir_all(&data_root)?;

        let passphrase = Arc::new(Mutex::new(passphrase.into()));
        let keystore = holochain_keystore::lair_keystore::spawn_lair_keystore_in_proc(
            &data_root.join("lair-keystore-config.yaml"),
            passphrase.clone(),
        )
        .await
        .context("open keystore (is LAIR_PASSPHRASE right?)")?;

        let conductor = ConductorBuilder::default()
            .passphrase(Some(passphrase))
            .config(conductor_config(&data_root, config))
            .with_keystore(keystore.clone())
            .build()
            .await
            .map_err(|e| anyhow!("start conductor: {e:?}"))?;
        let node = Self {
            conductor,
            keystore,
        };
        node.ensure_app_interface(config.app_ws_port).await?;
        Ok(node)
    }

    pub async fn admin(&self, request: AdminRequest) -> anyhow::Result<AdminResponse> {
        let response = AdminInterfaceApi::new(self.conductor.clone())
            .handle_request(Ok(request))
            .await
            .map_err(|e| anyhow!("admin call: {e:?}"))?;
        match response {
            AdminResponse::Error(e) => Err(anyhow!("admin call: {e:?}")),
            ok => Ok(ok),
        }
    }

    /// The conductor answers admin calls.
    pub async fn is_ready(&self) -> bool {
        self.admin(AdminRequest::ListApps {
            status_filter: None,
        })
        .await
        .is_ok()
    }

    pub fn keystore(&self) -> &MetaLairClient {
        &self.keystore
    }

    async fn ensure_app_interface(&self, port: u16) -> anyhow::Result<()> {
        if let AdminResponse::AppInterfacesListed(existing) =
            self.admin(AdminRequest::ListAppInterfaces).await?
        {
            if existing.iter().any(|i| i.port == port) {
                return Ok(());
            }
        }
        self.admin(AdminRequest::AttachAppInterface {
            port: Some(port),
            danger_bind_addr: None,
            allowed_origins: AllowedOrigins::Any,
            installed_app_id: None,
        })
        .await
        .map(|_| ())
    }
}

fn conductor_config(data_root: &Path, config: &Config) -> ConductorConfig {
    let mut advanced: serde_json::Value =
        serde_json::from_str(sp_core::network::GOSSIP_TUNING_JSON).unwrap_or_default();
    // An http:// relay only happens in local dev (scripts/dev-seed.sh); production is https.
    if config.relay_url.starts_with("http://") {
        advanced["irohTransport"] = serde_json::json!({ "relayAllowPlainText": true });
    }
    ConductorConfig {
        data_root_path: Some(data_root.to_path_buf().into()),
        keystore: KeystoreConfig::LairServerInProc { lair_root: None },
        network: NetworkConfig {
            bootstrap_url: url2::Url2::parse(&config.bootstrap_url),
            relay_url: url2::Url2::parse(&config.relay_url),
            // Full arc: the seed node holds everything (docs/NETWORK.md).
            target_arc_factor: 1,
            advanced: Some(advanced),
            ..NetworkConfig::default()
        },
        ..ConductorConfig::default()
    }
}
