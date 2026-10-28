//! The archivist (ADR 0031): an agent of this node that holds the commons (every public
//! space, ADR 0065), so projects stay available while their members are offline, and that
//! reads each listed space's counts for the feed index. Community nodes can run the same.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::anyhow;
use holo_hash::{ActionHashB64, AgentPubKeyB64};
use holochain::conductor::api::{AdminRequest, AppInfo, CellInfo};
use holochain::prelude::*;
use holochain_types::app::{AppBundleSource, InstallAppPayload};
use serde::Deserialize;

use super::spaces::{space_of_seed, Scoped};
use super::{Node, ProjectCard};

/// Every this many rounds (15 s each) every space is re-read: about every 15 minutes.
const FULL_EVERY: u64 = 60;
/// Every this many rounds the recently written spaces are: about once a minute.
const ACTIVE_EVERY: u64 = 4;

/// Mirrors the directory zome's `ListingView`.
#[derive(Debug, Deserialize)]
struct ListingView {
    id: ActionHash,
    creator: AgentPubKey,
    created_at: Timestamp,
    listing: Listing,
}

#[derive(Debug, Deserialize)]
struct Listing {
    network_seed: String,
    title: String,
    summary: String,
    mode: String,
    subtype: String,
    location_label: String,
    phase: String,
    tags: Vec<String>,
}

/// Mirrors the project zome's `ProjectView` (the fields the index needs).
#[derive(Debug, Deserialize)]
struct ProjectView {
    charter: Option<Charter>,
    stage: String,
    members: Vec<AgentPubKey>,
    demand: u32,
    opposition: u32,
    last_activity_at: Option<Timestamp>,
    updates: Vec<Update>,
}

/// An applied update (mirrors the project zome's `AppliedUpdate`; the card shows the newest).
#[derive(Debug, Deserialize)]
struct Update {
    body: String,
    at: Timestamp,
}

/// The profile field the feed shows (mirrors the identity zome's `Profile`).
#[derive(Debug, Deserialize)]
struct Profile {
    name: String,
}

/// The current charter fields the feed shows (passed edits applied).
#[derive(Debug, Deserialize)]
struct Charter {
    title: String,
    description: String,
    location_label: String,
}

impl Node {
    /// One pass over the listed public spaces (ADR 0065): all of them live in the archivist's
    /// one commons cell, so nothing is joined per project. A space is re-read when it is new
    /// to the index or was written to lately (its `recent.<hour>` bucket); every
    /// `FULL_EVERY` rounds all are. Returns the spaces indexed.
    pub async fn archive_round(&self, happ: &Path, round: u64) -> anyhow::Result<usize> {
        let app = self.archivist_app(happ).await?;
        let agent = app.agent_pub_key.clone();
        let directory = provisioned(&app, "directory")?;
        let commons = provisioned(&app, "project")?;
        let listings: Vec<ListingView> = self
            .call(&agent, directory.clone(), "identity", "list_projects", ())
            .await?;
        let full = round.is_multiple_of(FULL_EVERY);
        // Spaces written to this hour or the last; a writer marks a space once an hour, so a
        // marked space is re-read (once a minute) until its hour has passed.
        let active: BTreeSet<ActionHash> = if full || !round.is_multiple_of(ACTIVE_EVERY) {
            BTreeSet::new()
        } else {
            let hour = Timestamp::now().as_micros() / 3_600_000_000;
            let found: Vec<ActionHash> = self
                .call(&agent, commons.clone(), "project", "active_spaces", vec![hour - 1, hour])
                .await?;
            found.into_iter().collect()
        };
        let before = self.index.by_id().await;
        let mut cards = Vec::with_capacity(listings.len());
        for view in listings {
            let Some(space) = space_of_seed(&view.listing.network_seed) else {
                continue; // not a commons space (a listing from before ADR 0065)
            };
            let id = ActionHashB64::from(view.id.clone()).to_string();
            if let Some(card) = before.get(&id).filter(|_| !full && !active.contains(&space)) {
                cards.push(card.clone());
                continue;
            }
            cards.push(Box::pin(self.read_card(&agent, &directory, &commons, view, space)).await);
        }
        let count = cards.len();
        self.index.replace(cards).await;
        Ok(count)
    }

    /// A space's card: its listing, its live counts and its creator's name.
    async fn read_card(
        &self,
        agent: &AgentPubKey,
        directory: &CellId,
        commons: &CellId,
        view: ListingView,
        space: ActionHash,
    ) -> ProjectCard {
        let at = |input| Scoped {
            space: space.clone(),
            input,
        };
        let project: Option<ProjectView> = self
            .call(agent, commons.clone(), "project", "get_project", at(()))
            .await
            .unwrap_or(None);
        // The username comes with the card, so guests see it without a session.
        let profile: Option<Profile> = self
            .call(
                agent,
                directory.clone(),
                "identity",
                "get_profile",
                view.creator.clone(),
            )
            .await
            .unwrap_or(None);
        // Only the count is used; each message is skipped, not decoded.
        let messages: Vec<serde::de::IgnoredAny> = self
            .call(agent, commons.clone(), "discussion", "list_messages", at(()))
            .await
            .unwrap_or_default();
        let mut card = card(view, project);
        card.creator_name = profile.map(|p| p.name).unwrap_or_default();
        card.comment_count = u32::try_from(messages.len()).unwrap_or(u32::MAX);
        card
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

fn card(view: ListingView, project: Option<ProjectView>) -> ProjectCard {
    let created_at = view.created_at.as_micros();
    let mut l = view.listing;
    // The project's own network is the source of truth once decisions change it (ADR 0033).
    if let Some(p) = &project {
        if let Some(c) = &p.charter {
            l.title.clone_from(&c.title);
            l.summary = sp_core::project::summary_of(&c.description);
            l.location_label.clone_from(&c.location_label);
        }
        l.phase.clone_from(&p.stage);
    }
    ProjectCard {
        id: ActionHashB64::from(view.id).to_string(),
        network_seed: l.network_seed,
        creator: AgentPubKeyB64::from(view.creator).to_string(),
        creator_name: String::new(),
        title: l.title,
        summary: l.summary,
        mode: l.mode,
        subtype: l.subtype,
        location_label: l.location_label,
        phase: l.phase,
        tags: l.tags,
        members: project
            .as_ref()
            .map_or(1, |p| u32::try_from(p.members.len()).unwrap_or(u32::MAX)),
        demand: project.as_ref().map_or(0, |p| p.demand),
        opposition: project.as_ref().map_or(0, |p| p.opposition),
        created_at,
        last_activity_at: project
            .as_ref()
            .and_then(|p| p.last_activity_at)
            .map_or(created_at, |t| t.as_micros().max(created_at)),
        latest_update: project
            .and_then(|p| p.updates.into_iter().max_by_key(|u| u.at))
            .map(|u| u.body),
        comment_count: 0,
    }
}
