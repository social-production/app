//! Rules for one project (each project is its own clone of this DNA, ADR 0030).
//! The creator is named in the clone's network seed (`<creator key>.<nonce>`), so every member
//! agrees who it is and every node derives the same DNA hash from one plain string.
//! Membership and signals are links whose target is their author, so they need no entries.

use hdi::prelude::*;
use holo_hash::AgentPubKeyB64;
use sp_core::project::{check_fields, CharterFields, DESCRIPTION_MAX};

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Charter {
    pub title: String,
    pub description: String,
    pub mode: String,
    pub subtype: String,
    pub location_label: String,
    pub phase: String,
    pub tags: Vec<String>,
}

/// The tag byte of a `Signals` link.
pub const DEMAND: u8 = 0;
pub const OPPOSE: u8 = 1;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    Charter(Charter),
}

#[derive(Serialize, Deserialize)]
#[hdk_link_types]
pub enum LinkTypes {
    /// "charter" anchor → the charter's create action.
    ProjectCharter,
    /// "members" anchor → member agent.
    Members,
    /// "signals" anchor → signalling agent; tag `[DEMAND]` or `[OPPOSE]`.
    Signals,
}

/// The project's creator, from the network seed; `None` in the (unused) base cell.
pub fn creator() -> ExternResult<Option<AgentPubKey>> {
    Ok(creator_in_seed(&dna_info()?.modifiers.network_seed))
}

/// `<creator key>.<nonce>` → the creator key.
pub fn creator_in_seed(seed: &str) -> Option<AgentPubKey> {
    let (key, nonce) = seed.split_once('.')?;
    if nonce.is_empty() {
        return None;
    }
    AgentPubKeyB64::from_b64_str(key).ok().map(Into::into)
}

#[hdk_extern]
pub fn genesis_self_check(_data: GenesisSelfCheckData) -> ExternResult<ValidateCallbackResult> {
    // Open membrane (ADR 0006): anyone may join any project.
    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::CreateEntry(OpEntry::CreateEntry { app_entry, action })
        | FlatOp::CreateRecord(OpRecord::CreateEntry { app_entry, action }) => {
            let EntryTypes::Charter(c) = app_entry;
            by_creator(action.author(), || check_charter(&c))
        }
        FlatOp::CreateEntry(OpEntry::UpdateEntry {
            app_entry, action, ..
        })
        | FlatOp::CreateRecord(OpRecord::UpdateEntry {
            app_entry, action, ..
        }) => {
            let EntryTypes::Charter(c) = app_entry;
            by_creator(action.author(), || check_charter(&c))
        }
        FlatOp::CreateRecord(OpRecord::DeleteEntry { .. }) => invalid("charters are never deleted"),
        FlatOp::Link(OpLink::CreateLink { link_type, action })
        | FlatOp::CreateRecord(OpRecord::CreateLink { link_type, action }) => {
            validate_link(link_type, &action)
        }
        FlatOp::Link(OpLink::DeleteLink {
            original_action,
            action,
            ..
        }) => {
            if original_action.author() == action.author() {
                Ok(ValidateCallbackResult::Valid)
            } else {
                invalid("only the link's author can remove it")
            }
        }
        _ => Ok(ValidateCallbackResult::Valid),
    }
}

/// Until M3 (proposals) only the creator writes the charter.
fn by_creator(
    author: &AgentPubKey,
    then: impl FnOnce() -> Result<(), String>,
) -> ExternResult<ValidateCallbackResult> {
    if creator()?.as_ref() != Some(author) {
        return invalid("only the project's creator can write its charter");
    }
    match then() {
        Ok(()) => Ok(ValidateCallbackResult::Valid),
        Err(reason) => invalid(&reason),
    }
}

fn check_charter(c: &Charter) -> Result<(), String> {
    if c.description.trim().is_empty() || c.description.chars().count() > DESCRIPTION_MAX {
        return Err(format!(
            "description must be 1–{DESCRIPTION_MAX} characters"
        ));
    }
    check_fields(&CharterFields {
        title: &c.title,
        mode: &c.mode,
        subtype: &c.subtype,
        location_label: &c.location_label,
        phase: &c.phase,
        tags: &c.tags,
    })
}

fn validate_link(
    link_type: LinkTypes,
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    let target_is_author =
        action.target_address.clone().into_agent_pub_key().as_ref() == Some(action.author());
    match link_type {
        LinkTypes::ProjectCharter => by_creator(action.author(), || Ok(())),
        LinkTypes::Members if target_is_author => Ok(ValidateCallbackResult::Valid),
        LinkTypes::Signals if target_is_author => match action.tag.0.as_slice() {
            [DEMAND | OPPOSE] => Ok(ValidateCallbackResult::Valid),
            _ => invalid("a signal is demand or opposition"),
        },
        LinkTypes::Members | LinkTypes::Signals => {
            invalid("agents can only join or signal as themselves")
        }
    }
}

fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(reason.to_string()))
}
