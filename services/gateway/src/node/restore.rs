//! Bringing a person's app to Enabled, including a phrase restore (ADR 0035). A restore runs
//! in the background in the conductor; the app can't be enabled until it settles. A chain the
//! network doesn't hold never settles, so after `RESTORE_WAIT` the node starts a fresh chain.

use std::path::Path;
use std::time::Duration;

use anyhow::anyhow;
use holochain::conductor::api::{AdminRequest, AppInfo};
use holochain::prelude::*;
use holochain_types::app::{AppBundleSource, AppStatus, InstallAppPayload};

use super::Node;

/// The seed node holds the whole directory network (full arc), so an existing chain arrives
/// well within this.
const RESTORE_WAIT: Duration = Duration::from_secs(45);
const POLL: Duration = Duration::from_millis(500);

impl Node {
    /// Install (restoring from the network when asked), wait for any restore, and enable.
    pub(super) async fn install_and_enable(
        &self,
        id: &str,
        agent: &AgentPubKey,
        happ: &Path,
        restore: bool,
    ) -> anyhow::Result<AppInfo> {
        if self.find_app(id).await?.is_none() {
            self.install(id, agent, happ, restore).await?;
        }
        let started = tokio::time::Instant::now();
        loop {
            let app = self
                .find_app(id)
                .await?
                .ok_or_else(|| anyhow!("app {id} missing after install"))?;
            match &app.status {
                AppStatus::Enabled => return Ok(app),
                AppStatus::Disabled(_) => {
                    self.admin(AdminRequest::EnableApp {
                        installed_app_id: id.to_string(),
                    })
                    .await?;
                }
                AppStatus::AwaitingRestore if started.elapsed() < RESTORE_WAIT => {
                    tokio::time::sleep(POLL).await;
                }
                // No chain to restore (a new phrase, or one from a reset network): start fresh.
                AppStatus::AwaitingRestore | AppStatus::Unrecoverable(..) => {
                    tracing::info!(app = %id, "nothing to restore; starting a fresh chain");
                    self.admin(AdminRequest::UninstallApp {
                        installed_app_id: id.to_string(),
                        force: true,
                    })
                    .await?;
                    self.install(id, agent, happ, false).await?;
                }
                AppStatus::AwaitingMemproofs => {
                    return Err(anyhow!("app {id} is waiting for membrane proofs"))
                }
            }
        }
    }

    async fn install(
        &self,
        id: &str,
        agent: &AgentPubKey,
        happ: &Path,
        restore: bool,
    ) -> anyhow::Result<()> {
        let payload = InstallAppPayload {
            source: AppBundleSource::Path(happ.to_path_buf()),
            agent_key: Some(agent.clone()),
            installed_app_id: Some(id.to_string()),
            network_seed: None,
            roles_settings: None,
            ignore_genesis_failure: false,
            restore_from_dht: restore,
        };
        self.admin(AdminRequest::InstallApp(Box::new(payload)))
            .await
            .map(|_| ())
    }
}
