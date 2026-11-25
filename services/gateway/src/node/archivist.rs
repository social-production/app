//! The archivist (ADR 0031): an agent of this node that holds the commons (every public
//! space, ADR 0065), so projects stay available while their members are offline, and that
//! reads each listed space's counts for the feed index (`archive`, shared with the desktop
//! app). Community nodes can run the same.

use std::path::Path;

use anyhow::anyhow;
use archive::{Calls, Cells};
use holochain::conductor::api::{AdminRequest, AppInfo, CellInfo};
use holochain::prelude::*;
use holochain_types::app::{AppBundleSource, InstallAppPayload};

use super::Node;

/// The node's zome calls as the archivist's agent, for `archive`'s rounds.
struct AsArchivist<'n> {
    node: &'n Node,
    agent: AgentPubKey,
}

impl Calls for AsArchivist<'_> {
    async fn call<I, O>(&self, cell: &CellId, zome: &str, func: &str, payload: I) -> anyhow::Result<O>
    where
        I: serde::Serialize + std::fmt::Debug + Send,
        O: serde::de::DeserializeOwned + std::fmt::Debug + Send,
    {
        self.node
            .call(&self.agent, cell.clone(), zome, func, payload)
            .await
    }
}

impl Node {
    /// One pass over the listed public spaces into the feed index (`archive::archive_round`).
    pub async fn archive_round(&self, happ: &Path, round: u64) -> anyhow::Result<usize> {
        let app = self.archivist_app(happ).await?;
        let cells = Cells {
            directory: provisioned(&app, "directory")?,
            commons: provisioned(&app, "project")?,
        };
        let calls = AsArchivist {
            node: self,
            agent: app.agent_pub_key.clone(),
        };
        archive::archive_round(&calls, &cells, &self.index, round).await
    }

    /// Re-read every profile in the directory into the people index. Returns how many.
    pub async fn people_round(&self, happ: &Path) -> anyhow::Result<usize> {
        let app = self.archivist_app(happ).await?;
        let directory = provisioned(&app, "directory")?;
        let calls = AsArchivist {
            node: self,
            agent: app.agent_pub_key.clone(),
        };
        archive::people_round(&calls, &directory, &self.people).await
    }

    pub(super) async fn archivist_app(&self, happ: &Path) -> anyhow::Result<AppInfo> {
        let id = format!("archivist-{}", self.happ_version);
        if let Some(app) = self.find_app(&id).await? {
            return Ok(app);
        }
        // One archivist key for the life of the node, across hApp versions (ADR 0034).
        let agent = self.node_key("n-archivist").await?;
        let payload = InstallAppPayload {
            source: AppBundleSource::Path(happ.to_path_buf()),
            agent_key: Some(agent),
            installed_app_id: Some(id.clone()),
            network_seed: None,
            roles_settings: None,
            ignore_genesis_failure: false,
            restore_from_dht: false,
        };
        self.admin(AdminRequest::InstallApp(Box::new(payload)))
            .await?;
        self.admin(AdminRequest::EnableApp {
            installed_app_id: id.clone(),
        })
        .await?;
        tracing::info!(app = %id, "archivist installed");
        self.find_app(&id)
            .await?
            .ok_or_else(|| anyhow!("archivist missing after install"))
    }

    /// Call one function in each role's base cell, so the conductor compiles every zome
    /// before the first signup or project needs it (a cold compile takes about a minute).
    pub async fn warm_up(&self, happ: &Path) -> anyhow::Result<()> {
        let app = self.archivist_app(happ).await?;
        let agent = app.agent_pub_key.clone();
        let _: Vec<serde::de::IgnoredAny> = self
            .call(
                &agent,
                provisioned(&app, "directory")?,
                "identity",
                "founders",
                (),
            )
            .await?;
        let _: Vec<serde::de::IgnoredAny> = self
            .call(
                &agent,
                provisioned(&app, "project")?,
                "project",
                "active_spaces",
                Vec::<i64>::new(),
            )
            .await?;
        Ok(())
    }

    /// A zome call as the cell's own agent (author grant; no capability secret needed).
    pub(super) async fn call<I, O>(
        &self,
        agent: &AgentPubKey,
        cell: CellId,
        zome: &str,
        func: &str,
        payload: I,
    ) -> anyhow::Result<O>
    where
        I: serde::Serialize + std::fmt::Debug,
        O: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        self.conductor
            .easy_call_zome(agent, None, cell, zome, func, payload)
            .await
            .map_err(|e| anyhow!("{zome}/{func}: {e:?}"))
    }
}

pub(super) fn provisioned(app: &AppInfo, role: &str) -> anyhow::Result<CellId> {
    app.cell_info
        .get(role)
        .into_iter()
        .flatten()
        .find_map(|c| match c {
            CellInfo::Provisioned(c) => Some(c.cell_id.clone()),
            _ => None,
        })
        .ok_or_else(|| anyhow!("archivist has no {role} cell"))
}
