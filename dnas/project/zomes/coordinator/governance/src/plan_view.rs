//! The plan board for the participation tab (web's lifecycle `phaseTwo` / `phaseThree`
//! plans, with `criterionAssessments`, `overallApproval` and `leaderStatus`).

use hdk::prelude::*;
use project_integrity::{Plan, Proposal, ProposalKind};
use sp_governance::plans::{criteria, required_plan_kind, Leader, PlanKind, Score};
use sp_governance::{phases::Shape, required_votes};

use crate::apply::{self, Standing, State};
use crate::plans::{self, latest_ratings};
use crate::reads::{self, Found};
use crate::roll::Roll;
use crate::views::{tally_view, TallyView};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderStatus {
    Leading,
    Tied,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CriterionView {
    pub id: String,
    pub label: String,
    pub mine: Option<u8>,
    pub count: u32,
    pub average_x100: u32,
    pub distribution: [u32; 5],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlanView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub created_at: Timestamp,
    pub plan: Plan,
    pub tally: TallyView,
    /// Passed, and the recount of Full members' votes passes too.
    pub approved: bool,
    pub leader: Option<LeaderStatus>,
    pub criteria: Vec<CriterionView>,
    pub average_x100: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlanBoard {
    /// Newest first.
    pub plans: Vec<PlanView>,
    /// The kind of plan members can submit now (the current phase's), if any.
    pub open_kind: Option<PlanKind>,
    /// The kind whose approved leader the next phase move needs, if any.
    pub gate: Option<PlanKind>,
    /// That kind's leader: the plan the next phase move would cite.
    pub gate_leader: Option<ActionHash>,
    /// Members now, and the votes a new plan would need (web's quorum help).
    pub members: u32,
    pub required: u32,
}

pub fn board() -> ExternResult<PlanBoard> {
    let me = agent_info()?.agent_initial_pubkey;
    let mut standing = Standing::default();
    let roll = Roll::load()?;
    let state = apply::state(&mut standing, &roll)?;
    let mut plans = Vec::new();
    for p in reads::proposals()? {
        if let ProposalKind::Plan(plan) = &p.entry.kind {
            plans.push(view_of(&p, plan, &state, &mut standing, &me, &roll)?);
        }
    }
    plans.reverse();
    let shape = Shape {
        mode: state.mode(),
        subtype: state.subtype(),
        include_access: false,
    };
    let gate = required_plan_kind(shape, state.phase());
    let gate_leader = gate.and_then(|kind| match plans::leader(&state.approved, kind) {
        Leader::One(h) => Some(h),
        _ => None,
    });
    let members = u32::try_from(crate::calls::member_snapshot()?.len()).unwrap_or(u32::MAX);
    Ok(PlanBoard {
        plans,
        open_kind: PlanKind::for_phase(state.mode(), state.phase()),
        gate,
        gate_leader,
        members,
        required: required_votes(members),
    })
}

fn view_of(
    p: &Found<Proposal>,
    plan: &Plan,
    state: &State,
    standing: &mut Standing,
    me: &AgentPubKey,
    roll: &Roll,
) -> ExternResult<PlanView> {
    let rated = plans::rated(&p.hash, p.at, plan, standing)?;
    let mine: Vec<_> = latest_ratings(&p.hash)?
        .into_iter()
        .filter(|r| &r.author == me)
        .collect();
    let labels = criteria(plan.kind, plan.subtype_or_standard());
    let criteria = rated
        .iter()
        .zip(labels)
        .map(|((id, r), c)| CriterionView {
            id: (*id).to_string(),
            label: c.label.to_string(),
            mine: mine
                .iter()
                .find(|m| m.entry.criterion == *id)
                .and_then(|m| m.entry.rating),
            count: r.count,
            average_x100: r.average_x100(),
            distribution: r.distribution,
        })
        .collect();
    let score = Score::of(&rated.iter().map(|(_, r)| *r).collect::<Vec<_>>());
    let leader = match plans::leader(&state.approved, plan.kind) {
        Leader::One(h) if h == p.hash => Some(LeaderStatus::Leading),
        Leader::Tied(hs) if hs.contains(&p.hash) => Some(LeaderStatus::Tied),
        _ => None,
    };
    Ok(PlanView {
        id: p.hash.clone(),
        author: p.author.clone(),
        created_at: p.at,
        plan: plan.clone(),
        tally: tally_view(p, me, roll)?,
        approved: state.approved.iter().any(|a| a.hash == p.hash),
        leader,
        criteria,
        average_x100: score.average_x100(),
    })
}
