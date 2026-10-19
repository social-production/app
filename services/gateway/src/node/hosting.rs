//! Hosted agents (docs/IDENTITY.md): one installed app per person, and per-device
//! sessions where the browser's own signing key is granted access to that person's cells.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::anyhow;
use holo_hash::AgentPubKeyB64;
use holochain::conductor::api::{AdminRequest, AdminResponse, AppInfo, CellInfo};
use holochain::prelude::*;
use rand::RngCore;

use super::Node;

/// Installed-app id for a person's hosted agent on this hApp version. A new hApp (e.g. new
/// founders: a new directory DNA, so a new network) gets new app ids; old ones are removed at
/// start and people sign in again with their words (ADR 0028).
pub fn app_id(agent: &AgentPubKey, happ_version: &str) -> String {
    format!(
        "person-{}-{happ_version}",
        AgentPubKeyB64::from(agent.clone())
    )
}

/// Short, stable version: the first 8 hex chars of the sha256 of `bytes` (see `bundle::read`).
pub fn happ_version(bytes: &[u8]) -> String {
    use sha2::Digest;
    crate::hex::encode(&sha2::Sha256::digest(bytes)[..4])
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
        let id = app_id(agent, &self.happ_version);
        Box::pin(self.install_and_enable(&id, agent, happ, restore)).await
    }

    /// Grant `signing_key` every zome function on every cell of the person's app.
    pub async fn open_session(
        &self,
        app: &AppInfo,
        signing_key: AgentPubKey,
    ) -> anyhow::Result<Session> {
        let mut secret_bytes = [0_u8; CAP_SECRET_BYTES];
        rand::thread_rng().fill_bytes(&mut secret_bytes);
        let cells = cell_ids(app);
        self.grant(&cells, &signing_key, secret_bytes.into())
            .await?;
        let token = self.app_token(&app.installed_app_id).await?;
        Ok(Session {
            token,
            cap_secret: secret_bytes.to_vec(),
            cells,
        })
    }

    /// Extend a device's session to cells created since (joined or created projects).
    /// Only cells of the person's own app whose DNA is listed are granted.
    pub async fn grant_new_cells(
        &self,
        app: &AppInfo,
        signing_key: &AgentPubKey,
        cap_secret: CapSecret,
        dna_hashes: &[DnaHash],
    ) -> anyhow::Result<usize> {
        let cells: Vec<CellId> = cell_ids(app)
            .into_iter()
            .filter(|c| dna_hashes.contains(c.dna_hash()))
            .collect();
        self.grant(&cells, signing_key, cap_secret).await?;
        Ok(cells.len())
    }

    async fn grant(
        &self,
        cells: &[CellId],
        signing_key: &AgentPubKey,
        secret: CapSecret,
    ) -> anyhow::Result<()> {
        for cell_id in cells {
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
        Ok(())
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

    pub async fn find_app(&self, id: &str) -> anyhow::Result<Option<AppInfo>> {
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

#[cfg(test)]
mod tests {
    use super::{app_id, happ_version};
    use holochain::prelude::AgentPubKey;

    #[test]
    fn app_ids_change_with_the_happ() {
        let agent = AgentPubKey::from_raw_32(vec![7; 32]);
        let (v1, v2) = (happ_version(b"one"), happ_version(b"two"));
        assert_eq!(v1.len(), 8);
        assert_ne!(app_id(&agent, &v1), app_id(&agent, &v2));
        assert!(app_id(&agent, &v1).starts_with("person-uhCAk"));
    }
}
