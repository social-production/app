//! Rules for spaces (projects, events…). Public spaces share one network, the commons: this
//! DNA's base cell, where a space is its charter record and everything of the space hangs off
//! the charter's hash (ADR 0065). A private space is a clone whose seed names its creator
//! (ADR 0030, 0061) and holds that one space under the same rules.
//! Membership and signals are links from the space to their author, so they need no entries.
//! Decisions are Proposal → Vote → Outcome records (ADR 0009, ADR 0033). Values and their
//! importance votes are records too (ADR 0041); links to other projects are proposals (ADR 0042).
//! Activities and their roles, suggestions and ratings are records; taking a role is a link
//! (ADR 0056). A personal service's weekly hours are records and its bookings activities
//! (ADR 0077).

mod activity;
mod applied;
mod availability;
mod challenge;
mod charter;
mod governance;
mod history;
mod link;
mod links;
mod organizer;
mod place;
mod plan;
mod scope_item;
mod space;
mod value;

use hdi::prelude::*;

pub use activity::{Activity, ActivityRole, RoleSuggestion, SuggestionDecline};
pub use applied::{Applied, AppliedRequests, AppliedUpdate};
pub use availability::{AvailabilityRule, Booking};
pub use governance::{check_scope_kind, SNAPSHOT_MAX};
pub use history::{ActivityCompletion, ActivityRating};
pub use link::{LinkRequest, ProjectRef};
pub use organizer::{audience, organized, Organizer, OrganizerRole};
pub use place::Place;
pub use plan::{check as check_plan, Plan, PlanStage, Rating};
pub use scope_item::SEED_MAX as SCOPE_ITEM_SEED_MAX;
pub use space::{
    charter_at, creator, creator_in_seed, space_creator, space_in_seed, SPACE_SEED_WORD,
};
pub use value::{Importance, Value};

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Charter {
    pub title: String,
    pub description: String,
    pub mode: String,
    pub subtype: String,
    pub location_label: String,
    /// The place behind `location_label`, when it was picked with coordinates (ADR 0078).
    #[serde(default)]
    pub place: Option<Place>,
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
        /// With `location`: its place (ADR 0078); `None` there leaves the label without one.
        #[serde(default)]
        place: Option<Box<Place>>,
    },
    Update {
        body: String,
    },
    /// Leaving a plan phase cites the approved plan it goes ahead with (ADR 0039). web asks
    /// for a reason for every phase change (ADR 0040).
    Phase {
        to: String,
        plan: Option<ActionHash>,
        reason: String,
    },
    /// A production, distribution, operations or access plan; its approval is the vote.
    Plan(Plan),
    /// Back to an earlier planning phase, with web's required reason (ADR 0040). Whether `to`
    /// is earlier depends on the phase reached, so the fold checks it when applying.
    Return {
        to: String,
        reason: String,
    },
    /// One project's half of a link to another project (ADR 0042).
    Link(LinkRequest),
    /// Remove a member from a channel or community (web's `remove_member`, ADR 0068).
    RemoveMember {
        agent: AgentPubKey,
    },
    /// A service's request settings (web's settings change request, ADR 0076): voted in a
    /// collective service, saved at once by a personal service's creator.
    RequestSettings {
        enabled: bool,
        mode: String,
        off_schedule: bool,
        reason: String,
    },
}

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Proposal {
    /// The space it decides for: its charter's create hash (ADR 0065).
    pub space: ActionHash,
    pub kind: ProposalKind,
    /// The proposer's own `Members` link. Only a link request mirrored from another project
    /// may come from someone who isn't a member here (ADR 0042).
    pub membership: Option<ActionHash>,
    /// `Members` links counted as the population N (ADR 0033).
    pub members: Vec<ActionHash>,
    /// The proposer's `Organizer` record in an organizer-controlled event (ADR 0061).
    #[serde(default)]
    pub organizer: Option<ActionHash>,
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

/// Closes a proposal. Validators recompute the result from the cited votes, against the
/// snapshot plus the members the cited challenges add (ADR 0040).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Outcome {
    pub proposal: ActionHash,
    pub votes: Vec<ActionHash>,
    pub passed: bool,
    #[serde(default)]
    pub challenges: Vec<ActionHash>,
}

/// Memberships from before a proposal that its snapshot left out (ADR 0040).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Challenge {
    pub proposal: ActionHash,
    /// The challenger's own `Members` link.
    pub membership: ActionHash,
    pub omitted: Vec<ActionHash>,
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
    Rating(Rating),
    Challenge(Challenge),
    Value(Value),
    Importance(Importance),
    Activity(Activity),
    RoleSuggestion(RoleSuggestion),
    SuggestionDecline(SuggestionDecline),
    ActivityRating(ActivityRating),
    ActivityCompletion(ActivityCompletion),
    Organizer(Organizer),
    AvailabilityRule(AvailabilityRule),
}

#[derive(Serialize, Deserialize)]
#[hdk_link_types]
pub enum LinkTypes {
    /// "charter" anchor → the charter (private clones find their one space through it).
    ProjectCharter,
    /// Space → member agent.
    Members,
    /// Space → signalling agent; tag `[DEMAND]` or `[OPPOSE]`.
    Signals,
    /// Space → proposal.
    Proposals,
    ProposalToVotes,
    ProposalToOutcomes,
    /// Plan proposal → a member's criterion rating.
    ProposalToRatings,
    /// Proposal → a challenge to its count.
    ProposalToChallenges,
    /// Space → value.
    Values,
    /// Value → a member's importance vote.
    ValueToImportance,
    /// Space → activity.
    Activities,
    /// Activity → the member taking a role; tag `[role index]` + their `Members` link hash.
    Assignments,
    /// Activity → a role suggestion or a decline of one.
    ActivityToSuggestions,
    /// Activity → a participant's rating or completion check-in.
    ActivityToRatings,
    /// Space → an `Organizer` record (ADR 0061).
    Organizers,
    /// `recent.<UTC hour>` path → a space with activity in that hour, so indexes re-read only
    /// what changed (ADR 0065).
    Recent,
    /// A private community → a private event for its members; tag: the event's seed (ADR 0069).
    ScopeItems,
    /// A personal service → its creator's weekly hours; deleting the link removes them
    /// (ADR 0077).
    Availability,
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
            EntryTypes::Rating(r) => plan::validate_rating(&r, &action),
            EntryTypes::Challenge(c) => challenge::validate(&c, &action),
            EntryTypes::Value(v) => value::validate_value(&v, &action),
            EntryTypes::Importance(i) => value::validate_importance(&i, &action),
            EntryTypes::Activity(a) => activity::validate_activity(&a, &action),
            EntryTypes::RoleSuggestion(r) => activity::validate_suggestion(&r, &action),
            EntryTypes::SuggestionDecline(d) => activity::validate_decline(&d, &action),
            EntryTypes::ActivityRating(r) => history::validate_rating(&r, &action),
            EntryTypes::ActivityCompletion(c) => history::validate_completion(&c, &action),
            EntryTypes::Organizer(o) => organizer::validate(&o, &action),
            EntryTypes::AvailabilityRule(r) => availability::validate_rule(&r, &action),
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
            link_type,
            ..
        }) => {
            if original_action.author() != action.author() {
                invalid("only the link's author can remove it")
            } else if matches!(link_type, LinkTypes::Assignments) {
                activity::validate_leave(&original_action.base_address, action.timestamp())
            } else {
                Ok(ValidateCallbackResult::Valid)
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
