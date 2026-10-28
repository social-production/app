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
use space_scope::{enter, touch, In};

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
pub fn propose(call: In<ProposalKind>) -> ExternResult<ActionHash> {
    let kind = enter(call)?;
    let out = write::propose(kind)?;
    touch()?;
    Ok(out)
}

#[hdk_extern]
pub fn vote(call: In<VoteInput>) -> ExternResult<ActionHash> {
    let input = enter(call)?;
    let out = write::vote(input.proposal, input.choice)?;
    touch()?;
    Ok(out)
}

/// Rate one rubric criterion of a plan (1–5, or clear it). The latest rating counts.
#[hdk_extern]
pub fn rate(call: In<RateInput>) -> ExternResult<ActionHash> {
    let input = enter(call)?;
    let out = plan_write::rate(input)?;
    touch()?;
    Ok(out)
}

/// Write the Outcome for a settled proposal (anyone may; validators recompute it). Not named
/// `close`: hdk externs are unmangled symbols, and `close` would replace libc's on native builds.
#[hdk_extern]
pub fn close_proposal(call: In<ActionHash>) -> ExternResult<Option<ActionHash>> {
    let proposal = enter(call)?;
    let out = write::close(&proposal, None)?;
    touch()?;
    Ok(out)
}

/// Cite the members a proposal's count left out (ADR 0040); the proposal reopens with them.
#[hdk_extern]
pub fn challenge(call: In<ActionHash>) -> ExternResult<ActionHash> {
    let proposal = enter(call)?;
    let out = write::challenge(proposal)?;
    touch()?;
    Ok(out)
}

#[hdk_extern]
pub fn applied(call: In<()>) -> ExternResult<Applied> {
    enter(call)?;
    apply::applied()
}

/// web's "Add value" (Proposal phase only; a label already listed is refused).
#[hdk_extern]
pub fn add_value(call: In<String>) -> ExternResult<ActionHash> {
    let label = enter(call)?;
    let out = values::add(label)?;
    touch()?;
    Ok(out)
}

/// Rate a value's importance (1–10; Proposal phase only). The latest vote counts.
#[hdk_extern]
pub fn rate_value(call: In<ImportanceInput>) -> ExternResult<ActionHash> {
    let input = enter(call)?;
    let out = values::rate(input)?;
    touch()?;
    Ok(out)
}

/// Every value with its importance, oldest first.
#[hdk_extern]
pub fn list_values(call: In<()>) -> ExternResult<Vec<ValueView>> {
    enter(call)?;
    values::views(&mut apply::Standing::default())
}

/// Every proposal except plans, newest first.
#[hdk_extern]
pub fn list_proposals(call: In<()>) -> ExternResult<Vec<ProposalView>> {
    enter(call)?;
    views::list_proposals()
}

/// This project's halves of link requests, for the other projects' Links tabs (ADR 0042).
#[hdk_extern]
pub fn link_sides(call: In<()>) -> ExternResult<LinkSides> {
    enter(call)?;
    link_view::sides()
}

/// The Links tab: this project's halves paired with the other projects' `link_sides`.
#[hdk_extern]
pub fn list_links(call: In<Vec<LinkSides>>) -> ExternResult<LinksView> {
    let others = enter(call)?;
    link_view::view(&others)
}

/// Every plan with its assessment, and what the current phase needs.
#[hdk_extern]
pub fn list_plans(call: In<()>) -> ExternResult<PlanBoard> {
    enter(call)?;
    plan_view::board()
}
