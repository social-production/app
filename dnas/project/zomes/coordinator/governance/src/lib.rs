//! Project governance API (docs/plans/M3-governance.md, ADR 0033, 0039, 0040): propose, vote,
//! rate plans, challenge a count, add and rate project values (ADR 0041), read proposals and
//! plans with their tallies, links between projects (ADR 0042), and the applied state the
//! project view is built from.

mod apply;
mod calls;
mod count;
mod decide;
mod link_view;
mod link_write;
mod moves;
mod plan_view;
mod plan_write;
mod plans;
mod reads;
mod roll;
mod values;
mod views;
mod write;

use hdk::prelude::*;
use project_integrity::{Applied, ProposalKind, VoteChoice};

pub use link_view::{HalfView, LinkSides, LinkView, LinksView, RequestView};
pub use plan_view::{CriterionKind, CriterionView, LeaderStatus, PlanBoard, PlanView};
pub use plan_write::RateInput;
pub use values::{ImportanceInput, ValueView};
pub use views::{ProposalView, TallyView};

#[derive(Serialize, Deserialize, Debug)]
pub struct VoteInput {
    pub proposal: ActionHash,
    pub choice: VoteChoice,
}

/// File a proposal. A phase move's cited plan is filled in here (the current leader); plans,
/// returns and links come without their author's vote. A link request mirrored from another
/// project may be filed by someone who isn't a member here (ADR 0042).
#[hdk_extern]
pub fn propose(kind: ProposalKind) -> ExternResult<ActionHash> {
    write::propose(kind)
}

#[hdk_extern]
pub fn vote(input: VoteInput) -> ExternResult<ActionHash> {
    write::vote(input.proposal, input.choice)
}

/// Rate one rubric criterion of a plan (1–5, or clear it). The latest rating counts.
#[hdk_extern]
pub fn rate(input: RateInput) -> ExternResult<ActionHash> {
    plan_write::rate(input)
}

/// Write the Outcome for a settled proposal (anyone may; validators recompute it). Not named
/// `close`: hdk externs are unmangled symbols, and `close` would replace libc's on native builds.
#[hdk_extern]
pub fn close_proposal(proposal: ActionHash) -> ExternResult<Option<ActionHash>> {
    write::close(&proposal, None)
}

/// Cite the members a proposal's count left out (ADR 0040); the proposal reopens with them.
#[hdk_extern]
pub fn challenge(proposal: ActionHash) -> ExternResult<ActionHash> {
    write::challenge(proposal)
}

#[hdk_extern]
pub fn applied() -> ExternResult<Applied> {
    apply::applied()
}

/// web's "Add value" (Proposal phase only; a label already listed is refused).
#[hdk_extern]
pub fn add_value(label: String) -> ExternResult<ActionHash> {
    values::add(label)
}

/// Rate a value's importance (1–10; Proposal phase only). The latest vote counts.
#[hdk_extern]
pub fn rate_value(input: ImportanceInput) -> ExternResult<ActionHash> {
    values::rate(input)
}

/// Every value with its importance, oldest first.
#[hdk_extern]
pub fn list_values() -> ExternResult<Vec<ValueView>> {
    values::views(&mut apply::Standing::default())
}

/// Every proposal except plans, newest first.
#[hdk_extern]
pub fn list_proposals() -> ExternResult<Vec<ProposalView>> {
    views::list_proposals()
}

/// This project's halves of link requests, for the other projects' Links tabs (ADR 0042).
#[hdk_extern]
pub fn link_sides() -> ExternResult<LinkSides> {
    link_view::sides()
}

/// The Links tab: this project's halves paired with the other projects' `link_sides`.
#[hdk_extern]
pub fn list_links(others: Vec<LinkSides>) -> ExternResult<LinksView> {
    link_view::view(&others)
}

/// Every plan with its assessment, and what the current phase needs.
#[hdk_extern]
pub fn list_plans() -> ExternResult<PlanBoard> {
    plan_view::board()
}
