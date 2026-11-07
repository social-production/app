//! What the archivist reads to make a feed card (mirrors of the zomes' views) and the card it
//! makes. A help request's card also carries its time and sign-ups (ADR 0067).

use holo_hash::{ActionHashB64, AgentPubKeyB64};
use holochain::prelude::*;
use serde::{Deserialize, Serialize};

use super::ProjectCard;

/// Mirrors the directory zome's `ListingView`.
#[derive(Debug, Deserialize)]
pub struct ListingView {
    pub id: ActionHash,
    pub creator: AgentPubKey,
    pub created_at: Timestamp,
    pub listing: Listing,
}

#[derive(Debug, Deserialize)]
pub struct Listing {
    pub network_seed: String,
    pub title: String,
    pub summary: String,
    pub mode: String,
    pub subtype: String,
    pub location_label: String,
    pub phase: String,
    pub tags: Vec<String>,
}

/// Mirrors the project zome's `ProjectView` (the fields the index needs).
#[derive(Debug, Deserialize)]
pub struct ProjectView {
    pub charter: Option<Charter>,
    pub stage: String,
    pub members: Vec<AgentPubKey>,
    pub demand: u32,
    pub opposition: u32,
    pub last_activity_at: Option<Timestamp>,
    pub updates: Vec<Update>,
}

/// An applied update (mirrors the project zome's `AppliedUpdate`; the card shows the newest).
#[derive(Debug, Deserialize)]
pub struct Update {
    pub body: String,
    pub at: Timestamp,
}

/// Mirrors the discussion zome's `ModerationView` (the state only, ADR 0071).
#[derive(Debug, Deserialize)]
pub struct ModerationView {
    pub state: String,
}

/// The card's moderation field: nothing while the item is visible.
pub fn moderation_of(view: Option<ModerationView>) -> Option<String> {
    view.map(|m| m.state).filter(|s| s != "visible")
}

/// The profile field the feed shows (mirrors the identity zome's `Profile`).
#[derive(Debug, Deserialize)]
pub struct Profile {
    pub name: String,
}

/// The current charter fields the feed shows (passed edits applied).
#[derive(Debug, Deserialize)]
pub struct Charter {
    pub mode: String,
    pub title: String,
    pub description: String,
    pub location_label: String,
    /// ADR 0078; older charters have none.
    #[serde(default)]
    pub place: Option<super::markers::PlaceMirror>,
}

impl ProjectView {
    pub fn is_help_request(&self) -> bool {
        self.charter
            .as_ref()
            .is_some_and(|c| c.mode == "help-request")
    }
}

/// Mirrors the activities zome's `ActivityView` (what a help request's card and the map's
/// activity markers show, ADR 0067 and 0083).
#[derive(Debug, Deserialize)]
pub struct ActivityView {
    pub id: ActionHash,
    pub title: String,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
    pub online: bool,
    pub place: String,
    /// Older coordinators don't send it.
    #[serde(default)]
    pub location: Option<super::markers::PlaceMirror>,
    pub roles: Vec<RoleView>,
}

#[derive(Debug, Deserialize)]
pub struct RoleView {
    pub required: u32,
    pub maximum: Option<u32>,
    pub people: Vec<AgentPubKey>,
}

/// web's help-request card fields: when, who signed up, how many places there are.
#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HelpCard {
    pub needed_at: Option<i64>,
    pub ends_at: Option<i64>,
    pub signup_count: u32,
    /// Places across roles with a limit (web's `slotsNeeded`; "any number" roles add none).
    pub slots_needed: u32,
}

pub fn help_of(activities: &[ActivityView]) -> HelpCard {
    let Some(a) = activities.first() else {
        return HelpCard::default();
    };
    let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    HelpCard {
        needed_at: Some(a.starts_at.as_micros()),
        ends_at: Some(a.ends_at.as_micros()),
        signup_count: count(a.roles.iter().map(|r| r.people.len()).sum()),
        slots_needed: a.roles.iter().filter_map(|r| r.maximum).sum(),
    }
}

/// What the map keeps of an activity (ADR 0083): web's commitment counts are the people in
/// its roles and the sum of the roles' required counts.
pub fn card_activity(a: &ActivityView) -> super::markers::CardActivity {
    use super::markers::ActivityPlace;
    let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    super::markers::CardActivity {
        id: ActionHashB64::from(a.id.clone()).to_string(),
        title: a.title.clone(),
        starts_at: a.starts_at.as_micros(),
        ends_at: a.ends_at.as_micros(),
        label: a.place.clone(),
        place: match &a.location {
            Some(p) => p
                .physical()
                .map_or(ActivityPlace::Elsewhere, ActivityPlace::Own),
            None if a.online => ActivityPlace::Elsewhere,
            None => ActivityPlace::Parent,
        },
        committed: count(a.roles.iter().map(|r| r.people.len()).sum()),
        minimum: a.roles.iter().map(|r| r.required).sum(),
    }
}

pub fn card(view: ListingView, project: Option<ProjectView>) -> ProjectCard {
    let created_at = view.created_at.as_micros();
    let mut l = view.listing;
    let place = project
        .as_ref()
        .and_then(|p| p.charter.as_ref())
        .and_then(|c| c.place.as_ref())
        .and_then(super::markers::PlaceMirror::physical);
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
        help: None,
        scheduled_at: None,
        moderation: None,
        place,
        activities: Vec::new(),
    }
}
