//! Identity rules for the directory DNA (docs/GOVERNANCE.md#standing, docs/plans/M1-identity.md).
//! Every rule here is re-checked by every validator. Pure maths lives in `sp_governance`.

mod chain;
mod device;
mod founders;
mod listing;
mod profile;
mod standing;
mod vouch;

use hdi::prelude::*;

use chain::same_author;
pub(crate) use chain::{earlier_creates, invalid, prev_action, typed_entry};
pub use founders::{founder_agents, DirectoryProperties, Founder};

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Profile {
    pub name: String,
    pub bio: String,
    pub home_area: String,
    pub avatar: Option<Avatar>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Avatar {
    pub mime: String,
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

/// How the voucher shows it holds Full standing when the vouch is written.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum StandingProof {
    Founder,
    Grant(ActionHash),
}

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Vouch {
    pub vouchee: AgentPubKey,
    pub voucher_standing: StandingProof,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Evidence {
    /// Action hashes of `Vouch` records for the grantee.
    Vouches(Vec<ActionHash>),
    /// Proven from the grantee's own chain.
    SteadyActivity,
}

/// The grantee claims Full standing for itself; validators re-check the evidence.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct StandingGrant {
    pub evidence: Evidence,
}

/// One side of a device pairing. A link is complete when both agents wrote one.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct DeviceLink {
    pub primary: AgentPubKey,
    pub device: AgentPubKey,
}

/// A project in the directory (docs/plans/M2-projects.md). The project itself lives in its
/// own clone of the project DNA, joined with `network_seed` and the author as creator.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ProjectListing {
    pub network_seed: String,
    pub title: String,
    pub summary: String,
    pub mode: String,
    pub subtype: String,
    pub location_label: String,
    pub phase: String,
    pub tags: Vec<String>,
    /// Full members cite their standing and skip the New-member limit.
    pub creator_standing: Option<StandingProof>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    Profile(Profile),
    Vouch(Vouch),
    StandingGrant(StandingGrant),
    DeviceLink(DeviceLink),
    ProjectListing(ProjectListing),
}

#[derive(Serialize, Deserialize)]
#[hdk_link_types]
pub enum LinkTypes {
    AgentToProfile,
    AllPeople,
    VoucheeToVouches,
    AgentToGrants,
    AgentToDeviceLinks,
    AllProjects,
    AgentToListings,
}

#[hdk_extern]
pub fn genesis_self_check(_data: GenesisSelfCheckData) -> ExternResult<ValidateCallbackResult> {
    // Open membrane (ADR 0006): anyone may join the directory.
    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::CreateEntry(OpEntry::CreateEntry { app_entry, action })
        | FlatOp::CreateRecord(OpRecord::CreateEntry { app_entry, action }) => {
            validate_create(&app_entry, &action)
        }
        FlatOp::CreateEntry(OpEntry::UpdateEntry {
            app_entry, action, ..
        })
        | FlatOp::CreateRecord(OpRecord::UpdateEntry {
            app_entry, action, ..
        }) => match app_entry {
            EntryTypes::Profile(p) => profile::validate_update(&p, &action),
            EntryTypes::ProjectListing(l) => listing::validate_update(&l, &action),
            _ => invalid("only profiles and project listings can be updated"),
        },
        FlatOp::CreateRecord(OpRecord::DeleteEntry { action, .. }) => validate_delete(&action),
        FlatOp::Link(OpLink::CreateLink { link_type, action })
        | FlatOp::CreateRecord(OpRecord::CreateLink { link_type, action }) => {
            validate_create_link(link_type, &action)
        }
        FlatOp::Link(OpLink::DeleteLink {
            original_action,
            action,
            ..
        }) => same_author(
            original_action.author(),
            action.author(),
            "only the link author can remove it",
        ),
        _ => Ok(ValidateCallbackResult::Valid),
    }
}

fn validate_create(
    entry: &EntryTypes,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    match entry {
        EntryTypes::Profile(p) => profile::validate_fields(p),
        EntryTypes::Vouch(v) => vouch::validate_create(v, action),
        EntryTypes::StandingGrant(g) => standing::validate_create(g, action),
        EntryTypes::DeviceLink(d) => device::validate_create(d, action),
        EntryTypes::ProjectListing(l) => listing::validate_create(l, action),
    }
}

/// Vouches and device links can be withdrawn by their author; nothing else is deletable.
fn validate_delete(action: &TypedAction<DeleteData>) -> ExternResult<ValidateCallbackResult> {
    let original = must_get_valid_record(action.deletes_address.clone())?;
    let deletable = matches!(
        typed_entry(&original)?,
        Some(EntryTypes::Vouch(_) | EntryTypes::DeviceLink(_))
    );
    if !deletable {
        return invalid("only vouches and device links can be withdrawn");
    }
    same_author(
        original.action().author(),
        action.author(),
        "only the author can withdraw this",
    )
}

/// Links that describe an agent must be written by that agent.
fn validate_create_link(
    link_type: LinkTypes,
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    match link_type {
        LinkTypes::AgentToProfile | LinkTypes::AgentToGrants | LinkTypes::AgentToListings => {
            let base = action.base_address.clone().into_agent_pub_key();
            if base.as_ref() == Some(action.author()) {
                Ok(ValidateCallbackResult::Valid)
            } else {
                invalid("agents can only link their own profile, grants and listings")
            }
        }
        _ => Ok(ValidateCallbackResult::Valid),
    }
}
