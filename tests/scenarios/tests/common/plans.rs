//! Mirrors of the governance zome's plan types (tests can't link two integrity crates).

use holochain::prelude::*;
use serde::{Deserialize, Serialize};
use sp_governance::plans::PlanKind;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlanStage {
    pub title: String,
    pub details: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Plan {
    pub kind: PlanKind,
    pub title: String,
    pub description: String,
    pub demand_note: String,
    pub subtype: Option<String>,
    pub repository_url: Option<String>,
    pub location: Option<String>,
    pub recipients: Option<String>,
    pub stages: Vec<PlanStage>,
    /// Event plans only (ADR 0057).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<EventSchedule>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EventSchedule {
    pub mode: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub start_time: Option<String>,
    pub finish_time: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// A test mirror: built once per call, so the plan variant's size doesn't matter.
#[allow(clippy::large_enum_variant)]
pub enum ProposalKind {
    Phase {
        to: String,
        plan: Option<ActionHash>,
        reason: String,
    },
    Plan(Plan),
}

#[derive(Serialize, Debug)]
pub struct VoteInput {
    pub proposal: ActionHash,
    pub choice: &'static str,
}

#[derive(Serialize, Debug)]
pub struct RateInput {
    pub proposal: ActionHash,
    pub criterion: String,
    pub rating: Option<u8>,
}

#[derive(Deserialize, Debug)]
pub struct Tally {
    pub yes: u32,
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct Criterion {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub mine: Option<u8>,
    pub count: u32,
}

#[derive(Deserialize, Debug)]
pub struct PlanView {
    pub id: ActionHash,
    pub tally: Tally,
    pub approved: bool,
    pub leader: Option<String>,
    pub criteria: Vec<Criterion>,
}

#[derive(Deserialize, Debug)]
pub struct PlanBoard {
    pub plans: Vec<PlanView>,
    pub open_kind: Option<PlanKind>,
    pub gate: Option<PlanKind>,
    pub gate_leader: Option<ActionHash>,
}
