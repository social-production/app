//! What passed proposals change, as the governance coordinator reports it.

use hdi::prelude::*;

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
