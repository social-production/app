//! Project listings: the directory's index of projects (each project lives in its own clone).

use hdk::prelude::*;
use identity_integrity::{EntryTypes, LinkTypes, ProjectListing, UnitEntryTypes};
use sp_governance::projects::{
    check_new_member_project_cap, NEW_PROJECT_CAP, NEW_PROJECT_WINDOW_US,
};

use crate::guest_error;
use crate::records::{action_targets, decode, latest, links};
use crate::standing::{own_entry_times, own_proof};

const ALL_PROJECTS: &str = "all_projects";

/// The listing fields the UI sends; the zome adds the creator's standing proof.
#[derive(Serialize, Deserialize, Debug)]
pub struct ListingInput {
    pub network_seed: String,
    pub title: String,
    pub summary: String,
    pub mode: String,
    pub subtype: String,
    pub location_label: String,
    pub phase: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListingView {
    /// The original listing action: the project's stable id in the directory.
    pub id: ActionHash,
    pub creator: AgentPubKey,
    pub created_at: Timestamp,
    pub listing: ProjectListing,
}

/// How many more projects the caller may start this week (`None`: no limit).
#[derive(Serialize, Deserialize, Debug)]
pub struct Allowance {
    pub remaining: Option<u32>,
}

pub fn create(input: ListingInput) -> ExternResult<ActionHash> {
    let creator_standing = own_proof()?;
    if creator_standing.is_none() {
        let earlier = own_entry_times(UnitEntryTypes::ProjectListing)?;
        check_new_member_project_cap(&earlier, sys_time()?.as_micros()).map_err(guest_error)?;
    }
    let hash = create_entry(&EntryTypes::ProjectListing(to_listing(
        input,
        creator_standing,
    )))?;
    let me = agent_info()?.agent_initial_pubkey;
    create_link(
        Path::from(ALL_PROJECTS).path_entry_hash()?,
        hash.clone(),
        LinkTypes::AllProjects,
        (),
    )?;
    create_link(me, hash.clone(), LinkTypes::AgentToListings, ())?;
    Ok(hash)
}

pub fn update(id: ActionHash, input: ListingInput) -> ExternResult<ActionHash> {
    let Some(record) = latest(id.clone())? else {
        return Err(guest_error("That project isn't listed."));
    };
    let standing = decode::<ProjectListing>(&record)?.and_then(|l| l.creator_standing);
    update_entry(id, &EntryTypes::ProjectListing(to_listing(input, standing)))
}

/// Every listed project, newest first, each at its latest version.
pub fn list() -> ExternResult<Vec<ListingView>> {
    let mut found = links(
        Path::from(ALL_PROJECTS).path_entry_hash()?,
        LinkTypes::AllProjects,
    )?;
    found.sort_by_key(|l| std::cmp::Reverse(l.timestamp));
    let mut out = Vec::new();
    for id in action_targets(found) {
        if out.iter().any(|v: &ListingView| v.id == id) {
            continue;
        }
        if let Some(view) = view(id)? {
            out.push(view);
        }
    }
    Ok(out)
}

pub fn get(id: ActionHash) -> ExternResult<Option<ListingView>> {
    view(id)
}

pub fn allowance() -> ExternResult<Allowance> {
    if own_proof()?.is_some() {
        return Ok(Allowance { remaining: None });
    }
    let now = sys_time()?.as_micros();
    let recent = own_entry_times(UnitEntryTypes::ProjectListing)?
        .into_iter()
        .filter(|t| *t > now - NEW_PROJECT_WINDOW_US)
        .count();
    let left = NEW_PROJECT_CAP.saturating_sub(recent);
    Ok(Allowance {
        remaining: Some(u32::try_from(left).unwrap_or(0)),
    })
}

fn view(id: ActionHash) -> ExternResult<Option<ListingView>> {
    let Some(original) = get_original(&id)? else {
        return Ok(None);
    };
    let Some(record) = latest(id.clone())? else {
        return Ok(None);
    };
    Ok(
        decode::<ProjectListing>(&record)?.map(|listing| ListingView {
            id,
            creator: original.action().author().clone(),
            created_at: original.action().timestamp(),
            listing,
        }),
    )
}

fn get_original(id: &ActionHash) -> ExternResult<Option<Record>> {
    hdk::prelude::get(id.clone(), GetOptions::default())
}

fn to_listing(
    input: ListingInput,
    creator_standing: Option<identity_integrity::StandingProof>,
) -> ProjectListing {
    ProjectListing {
        network_seed: input.network_seed,
        title: input.title,
        summary: input.summary,
        mode: input.mode,
        subtype: input.subtype,
        location_label: input.location_label,
        phase: input.phase,
        tags: input.tags,
        creator_standing,
    }
}
