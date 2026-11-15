//! What passed proposals change, as the governance coordinator reports it.

use hdi::prelude::*;
use sp_governance::phases::PROPOSAL;

/// Validation reads only the charter, where a proposal's mode stays `proposal` whatever its
/// plans made it (ADR 0059): what it became is replayed by the coordinator, never here. So a
/// rule for one type lets a proposal through as that type, and the coordinator refuses the
/// write when the proposal is not (or not yet) that type.
#[must_use]
pub fn typed_as<'a>(mode: &'a str, could_be: &'a str) -> &'a str {
    if mode == PROPOSAL {
        could_be
    } else {
        mode
    }
}

/// Whether a space of `mode` is, or as a proposal may have become, `wanted`.
#[must_use]
pub fn may_be(mode: &str, wanted: &str) -> bool {
    typed_as(mode, wanted) == wanted
}

/// What the governance coordinator reports as applied, for the project coordinator's view.
/// Defined here because both coordinators depend on this crate, never on each other.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Applied {
    /// Passed edits merged in order: the latest value of each field.
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    /// The place the edit that set `location` gave it, if any (ADR 0078).
    #[serde(default)]
    pub place: Option<crate::Place>,
    /// The phase reached through passed `Phase` proposals, if any.
    pub phase: Option<String>,
    /// The subtype chosen by the production or operations plan (ADR 0039), if any.
    pub subtype: Option<String>,
    /// The type a proposal's leading plan gave it (ADR 0059): `productive`,
    /// `collective-service` or `event`; none while it is still a proposal, or for a space
    /// that was founded with its type.
    #[serde(default)]
    pub mode: Option<String>,
    /// The type a proposal's leading plan would give it if it left the Plan phase now.
    #[serde(default)]
    pub leading_mode: Option<String>,
    pub updates: Vec<AppliedUpdate>,
    pub last_decision_at: Option<Timestamp>,
    /// Members removed by passed `RemoveMember` proposals, and when (ADR 0068).
    #[serde(default)]
    pub removed: Vec<(AgentPubKey, Timestamp)>,
    /// The request settings the newest passed `RequestSettings` proposal set (ADR 0076).
    #[serde(default)]
    pub requests: Option<AppliedRequests>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AppliedRequests {
    pub enabled: bool,
    pub mode: String,
    pub off_schedule: bool,
    pub at: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AppliedUpdate {
    pub proposal: ActionHash,
    pub author: AgentPubKey,
    pub body: String,
    pub at: Timestamp,
}
