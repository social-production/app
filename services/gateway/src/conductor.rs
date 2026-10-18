//! The gateway's admin connection to the local conductor (localhost only).
//! M0 hosts a single shared demo agent; M1 replaces it with one hosted agent per user.

use std::collections::BTreeSet;
use std::time::Duration;

use anyhow::{anyhow, Context};
use holochain_client::{
    AdminWebsocket, AgentPubKey, AllowedOrigins, AppInfo, CellId, CellInfo, InstallAppPayload,
    IssueAppAuthenticationTokenPayload,
};
use holochain_types::app::AppBundleSource;
use holochain_zome_types::capability::GrantZomeCallCapabilityGrant;
use holochain_zome_types::prelude::{
    CapSecret, GrantConstraint, GrantZomeCallCapabilityPayload, GrantedFunctions, ZomeCallGrant,
    CAP_SECRET_BYTES,
};
use rand::RngCore;

use crate::config::Config;

pub const DEMO_APP_ID: &str = "sp-demo";
const CONNECT_ATTEMPTS: u32 = 120;

#[derive(Clone)]
pub struct Conductor {
    admin: AdminWebsocket,
}

impl std::fmt::Debug for Conductor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Conductor")
    }
}

impl Conductor {
    /// Connect (waiting for the conductor to boot), then make sure the demo app is
    /// installed and the app interface the `/ws` proxy forwards to exists.
    pub async fn start(config: &Config) -> anyhow::Result<Self> {
        let admin = connect_admin(config.admin_port).await?;
        let conductor = Self { admin };
        conductor.ensure_demo_app(config).await?;
        conductor.ensure_app_interface(config.app_ws_port).await?;
        Ok(conductor)
    }

    pub async fn is_ready(&self) -> bool {
        self.demo_app().await.is_ok()
    }

    /// Grant `signing_key` the right to call every zome on every demo cell, and issue a
    /// single-use app-websocket token. See docs/IDENTITY.md#zome-call-signing.
    pub async fn open_session(&self, signing_key: AgentPubKey) -> anyhow::Result<Session> {
        let app = self.demo_app().await?;
        let cells = cell_ids(&app);
        let secret = random_secret();
        for cell_id in &cells {
            self.grant(cell_id.clone(), signing_key.clone(), secret)
                .await?;
        }
        let issued = self
            .admin
            .issue_app_auth_token(IssueAppAuthenticationTokenPayload {
                installed_app_id: DEMO_APP_ID.to_string(),
                expiry_seconds: 60,
                single_use: true,
            })
            .await
            .map_err(|e| anyhow!("issue token: {e:?}"))?;
        Ok(Session {
            token: issued.token,
            cap_secret: secret.as_ref().to_vec(),
            cells,
        })
    }

    async fn grant(
        &self,
        cell_id: CellId,
        key: AgentPubKey,
        secret: CapSecret,
    ) -> anyhow::Result<()> {
        let cap_grant = GrantZomeCallCapabilityGrant {
            tag: "browser-session".to_string(),
            constraint: GrantConstraint::Assigned {
                secret,
                assignees: BTreeSet::from([key]),
            },
            grant: ZomeCallGrant {
                functions: GrantedFunctions::All,
            },
        };
        self.admin
            .grant_zome_call_capability(GrantZomeCallCapabilityPayload { cell_id, cap_grant })
            .await
            .map(|_| ())
            .map_err(|e| anyhow!("grant capability: {e:?}"))
    }

    async fn demo_app(&self) -> anyhow::Result<AppInfo> {
        let apps = self
            .admin
            .list_apps(None)
            .await
            .map_err(|e| anyhow!("list apps: {e:?}"))?;
        apps.into_iter()
            .find(|app| app.installed_app_id == DEMO_APP_ID)
            .context("demo app not installed")
    }

    async fn ensure_demo_app(&self, config: &Config) -> anyhow::Result<()> {
        if self.demo_app().await.is_ok() {
            return Ok(());
        }
        tracing::info!(path = %config.happ_path.display(), "installing demo app");
        self.admin
            .install_app(InstallAppPayload {
                source: AppBundleSource::Path(config.happ_path.clone()),
                agent_key: None,
                installed_app_id: Some(DEMO_APP_ID.to_string()),
                network_seed: None,
                roles_settings: None,
                ignore_genesis_failure: false,
                restore_from_dht: false,
            })
            .await
            .map_err(|e| anyhow!("install app: {e:?}"))?;
        self.admin
            .enable_app(DEMO_APP_ID.to_string())
            .await
            .map_err(|e| anyhow!("enable app: {e:?}"))?;
        Ok(())
    }

    async fn ensure_app_interface(&self, port: u16) -> anyhow::Result<()> {
        let existing = self
            .admin
            .list_app_interfaces()
            .await
            .map_err(|e| anyhow!("list app interfaces: {e:?}"))?;
        if existing.iter().any(|i| i.port == port) {
            return Ok(());
        }
        self.admin
            .attach_app_interface(port, None, AllowedOrigins::Any, None)
            .await
            .map(|_| ())
            .map_err(|e| anyhow!("attach app interface: {e:?}"))
    }
}

#[derive(Debug)]
pub struct Session {
    pub token: Vec<u8>,
    pub cap_secret: Vec<u8>,
    pub cells: Vec<CellId>,
}

async fn connect_admin(port: u16) -> anyhow::Result<AdminWebsocket> {
    for attempt in 1..=CONNECT_ATTEMPTS {
        match AdminWebsocket::connect(("127.0.0.1", port), Some("sp-gateway".to_string())).await {
            Ok(admin) => return Ok(admin),
            Err(e) if attempt == CONNECT_ATTEMPTS => {
                return Err(anyhow!("conductor admin port {port} unreachable: {e:?}"))
            }
            Err(_) => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    }
    unreachable!("loop returns on the last attempt")
}

fn cell_ids(app: &AppInfo) -> Vec<CellId> {
    app.cell_info
        .values()
        .flatten()
        .filter_map(|cell| match cell {
            CellInfo::Provisioned(c) => Some(c.cell_id.clone()),
            CellInfo::Cloned(c) => Some(c.cell_id.clone()),
            CellInfo::Stem(_) => None, // not yet materialised (deferred memproofs)
        })
        .collect()
}

fn random_secret() -> CapSecret {
    let mut bytes = [0_u8; CAP_SECRET_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.into()
}
