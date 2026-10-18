//! Hosted agents (docs/IDENTITY.md): one installed app per person, and per-device
//! sessions where the browser's own signing key is granted access to that person's cells.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::anyhow;
use holo_hash::AgentPubKeyB64;
use holochain::conductor::api::{AdminRequest, AdminResponse, AppInfo, CellInfo};
use holochain::prelude::*;
use holochain_types::app::{AppBundleSource, InstallAppPayload};
use rand::RngCore;

use super::Node;

/// Installed-app id for a person's hosted agent.
pub fn app_id(agent: &AgentPubKey) -> String {
    format!("person-{}", AgentPubKeyB64::from(agent.clone()))
}

#[derive(Debug)]
pub struct Session {
    pub token: Vec<u8>,
    pub cap_secret: Vec<u8>,
    pub cells: Vec<CellId>,
}

impl Node {
    /// Install and enable the person's app on first use. `restore` rebuilds an existing
    /// person's chain from the network (recovery-phrase restore on a new node).
    pub async fn ensure_hosted(
        &self,
        agent: &AgentPubKey,
        happ: &Path,
        restore: bool,
    ) -> anyhow::Result<AppInfo> {
        let id = app_id(agent);
        if let Some(app) = self.find_app(&id).await? {
            return Ok(app);
        }
        let payload = InstallAppPayload {
            source: AppBundleSource::Path(happ.to_path_buf()),
            agent_key: Some(agent.clone()),
            installed_app_id: Some(id.clone()),
            network_seed: None,
            roles_settings: None,
            ignore_genesis_failure: false,
            restore_from_dht: restore,
        };
        self.admin(AdminRequest::InstallApp(Box::new(payload)))
            .await?;
        self.admin(AdminRequest::EnableApp {
            installed_app_id: id.clone(),
        })
        .await?;
        self.find_app(&id)
            .await?
            .ok_or_else(|| anyhow!("app {id} missing after install"))
    }

    /// Grant `signing_key` every zome function on every cell of the person's app.
    pub async fn open_session(
        &self,
        app: &AppInfo,
        signing_key: AgentPubKey,
    ) -> anyhow::Result<Session> {
        let mut secret_bytes = [0_u8; CAP_SECRET_BYTES];
        rand::thread_rng().fill_bytes(&mut secret_bytes);
        let secret: CapSecret = secret_bytes.into();
        let cells = cell_ids(app);
        for cell_id in &cells {
            let cap_grant = GrantZomeCallCapabilityGrant {
                tag: "device-session".into(),
                constraint: GrantConstraint::Assigned {
                    secret,
                    assignees: BTreeSet::from([signing_key.clone()]),
                },
                grant: ZomeCallGrant {
                    functions: GrantedFunctions::All,
                },
            };
            let payload = GrantZomeCallCapabilityPayload {
                cell_id: cell_id.clone(),
                cap_grant,
            };
            self.admin(AdminRequest::GrantZomeCallCapability(Box::new(payload)))
                .await?;
        }
        let token = self.app_token(&app.installed_app_id).await?;
        Ok(Session {
            token,
            cap_secret: secret_bytes.to_vec(),
            cells,
        })
    }

    /// Single-use, short-lived token to open an app websocket for this app.
    pub async fn app_token(&self, installed_app_id: &str) -> anyhow::Result<Vec<u8>> {
        let request = AdminRequest::IssueAppAuthenticationToken(
            holochain::conductor::api::IssueAppAuthenticationTokenPayload {
                installed_app_id: installed_app_id.to_string(),
                expiry_seconds: 60,
                single_use: true,
            },
        );
        match self.admin(request).await? {
            AdminResponse::AppAuthenticationTokenIssued(issued) => Ok(issued.token),
            other => Err(anyhow!("unexpected token response: {other:?}")),
        }
    }

    async fn find_app(&self, id: &str) -> anyhow::Result<Option<AppInfo>> {
        match self
            .admin(AdminRequest::ListApps {
                status_filter: None,
            })
            .await?
        {
            AdminResponse::AppsListed(apps) => {
                Ok(apps.into_iter().find(|a| a.installed_app_id == id))
            }
            other => Err(anyhow!("unexpected list response: {other:?}")),
        }
    }
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
