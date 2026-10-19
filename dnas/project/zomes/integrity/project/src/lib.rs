//! Rules for one project (each project is its own clone of this DNA, ADR 0030).
//! The creator is named in the clone's network seed (`<creator key>.<nonce>`), so every member
//! agrees who it is and every node derives the same DNA hash from one plain string.
//! Membership and signals are links whose target is their author, so they need no entries.
//! Decisions are Proposal → Vote → Outcome records (ADR 0009, ADR 0033).

mod charter;
mod governance;
mod links;

use hdi::prelude::*;
use holo_hash::AgentPubKeyB64;

pub use governance::SNAPSHOT_MAX;

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Charter {
    pub title: String,
    pub description: String,
    pub mode: String,
    pub subtype: String,
    pub location_label: String,
    /// The phase the project was created in; later phases come from passed proposals.
    pub phase: String,
    pub tags: Vec<String>,
}

/// What a proposal decides. An edit stores only what changes (as web does), so two passed
/// edits of different fields never undo each other.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ProposalKind {
    Edit {
        title: Option<String>,
        description: Option<String>,
        location: Option<String>,
    },
    Update {
        body: String,
    },
    Phase {
        to: String,
    },
}

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Proposal {
    pub kind: ProposalKind,
    /// The proposer's own `Members` link.
    pub membership: ActionHash,
    /// `Members` links counted as the population N (ADR 0033).
    pub members: Vec<ActionHash>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoteChoice {
    Yes,
    No,
    /// Recorded but not counted (web's "neutral").
    Neutral,
}

/// Final once cast: one per agent per proposal (ADR 0033).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Vote {
    pub proposal: ActionHash,
    pub membership: ActionHash,
    pub choice: VoteChoice,
}

/// Closes a proposal. Validators recompute the result from the cited votes.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Outcome {
    pub proposal: ActionHash,
    pub votes: Vec<ActionHash>,
    pub passed: bool,
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
    Proposal(Proposal),
    Vote(Vote),
    Outcome(Outcome),
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
    /// "proposals" anchor → proposal.
    Proposals,
    ProposalToVotes,
    ProposalToOutcomes,
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
        | FlatOp::CreateRecord(OpRecord::CreateEntry { app_entry, action }) => match app_entry {
            EntryTypes::Charter(c) => charter::validate_create(&c, action.author()),
            EntryTypes::Proposal(p) => governance::validate_proposal(&p, &action),
            EntryTypes::Vote(v) => governance::validate_vote(&v, &action),
            EntryTypes::Outcome(o) => governance::validate_outcome(&o, &action),
        },
        // Charters change through proposals; votes are final; outcomes are history.
        FlatOp::CreateEntry(OpEntry::UpdateEntry { .. })
        | FlatOp::CreateRecord(OpRecord::UpdateEntry { .. }) => {
            invalid("project records are never updated")
        }
        FlatOp::CreateRecord(OpRecord::DeleteEntry { .. }) => {
            invalid("project records are never deleted")
        }
        FlatOp::Link(OpLink::CreateLink { link_type, action })
        | FlatOp::CreateRecord(OpRecord::CreateLink { link_type, action }) => {
            links::validate_create(link_type, &action)
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

pub(crate) fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(reason.to_string()))
}

/// Decode a record's entry using its declared entry type (never by guessing from bytes).
pub(crate) fn typed_entry(record: &Record) -> ExternResult<Option<EntryTypes>> {
    let Some(entry) = record.entry().as_option() else {
        return Ok(None);
    };
    let Some(EntryType::App(def)) = record.action().entry_type() else {
        return Ok(None);
    };
    EntryTypes::deserialize_from_type(def.zome_index, def.entry_index, entry)
}

/// What the governance coordinator reports as applied, for the project coordinator's view.
/// Defined here because both coordinators depend on this crate, never on each other.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Applied {
    /// Passed edits merged in order: the latest value of each field.
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    /// The phase reached through passed `Phase` proposals, if any.
    pub phase: Option<String>,
    pub updates: Vec<AppliedUpdate>,
    pub last_decision_at: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AppliedUpdate {
    pub proposal: ActionHash,
    pub author: AgentPubKey,
    pub body: String,
    pub at: Timestamp,
}
