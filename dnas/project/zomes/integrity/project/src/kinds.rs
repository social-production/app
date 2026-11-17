//! What a proposal decides (`ProposalKind`), split out of `lib.rs` for size.

use hdi::prelude::*;

use crate::{LinkRequest, Place, Plan, SoftwareRequest};

/// A scope's picture or cover after an edit (ADR 0109): a directory `ScopeImage`, or none.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ImageChange {
    Set(EntryHash),
    Remove,
}

/// What a proposal decides. An edit stores only what changes (as web does).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)] // boxing the plan would change every caller
pub enum ProposalKind {
    Edit {
        title: Option<String>,
        description: Option<String>,
        location: Option<String>,
        /// With `location`: its place (ADR 0078); `None` there leaves the label without one.
        #[serde(default)]
        place: Option<Box<Place>>,
        /// A scope's new picture or cover, or its removal (ADR 0109).
        #[serde(default)]
        picture: Option<ImageChange>,
        #[serde(default)]
        cover: Option<ImageChange>,
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
    /// Request settings (ADR 0076): voted in a collective service, saved by a personal one's.
    RequestSettings {
        enabled: bool,
        mode: String,
        off_schedule: bool,
        reason: String,
    },
    /// A software project's pull request, merge, merge capability or repository (ADR 0086).
    Software(SoftwareRequest),
    /// Close this and continue the work as a new proposal (ADR 0106).
    Convert {
        reason: String,
    },
}
