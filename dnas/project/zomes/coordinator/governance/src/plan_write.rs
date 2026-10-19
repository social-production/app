//! Submitting and rating plans, with web's readable refusals (web-backend `projects_plans.py`).

use hdk::prelude::*;
use project_integrity::{check_plan, EntryTypes, LinkTypes, Plan, ProposalKind, Rating};
use sp_governance::plans::{criteria, rating_ok};

use crate::apply::State;
use crate::calls;
use crate::reads;
use crate::write::guest;

/// web's `_assert_plan_type_allowed` and `_assert_plan_phase_is_current`. Production and
/// operations plans always name a subtype (the project's, unless the author picks one), so
/// the rubric and the subtype a leader chooses never depend on later state.
pub fn prepare(state: &State, mut plan: Plan) -> ExternResult<Plan> {
    if !plan.kind.allowed(state.mode()) {
        return Err(guest("This project mode does not take this kind of plan."));
    }
    if state.phase() != plan.kind.phase() {
        return Err(guest(
            "Plans can only be submitted during the matching lifecycle phase.",
        ));
    }
    plan.subtype = if plan.kind.sets_subtype() {
        Some(plan.subtype.unwrap_or_else(|| state.subtype().to_string()))
    } else {
        None
    };
    // Fail early with integrity's own reason.
    check_plan(&plan).map_err(guest)?;
    Ok(plan)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RateInput {
    pub proposal: ActionHash,
    pub criterion: String,
    /// 1–5, or `None` to clear.
    pub rating: Option<u8>,
}

pub fn rate(input: RateInput) -> ExternResult<ActionHash> {
    let membership =
        calls::my_membership()?.ok_or_else(|| guest("Join the project to rate its plans."))?;
    let Some(found) = reads::proposal(&input.proposal)? else {
        return Err(guest("This plan hasn't reached your node yet."));
    };
    let ProposalKind::Plan(plan) = found.entry.kind else {
        return Err(guest("Only plans are rated."));
    };
    if !criteria(plan.kind, plan.subtype_or_standard())
        .iter()
        .any(|c| c.id == input.criterion)
    {
        return Err(guest("Unknown plan criterion."));
    }
    if input.rating.is_some_and(|r| !rating_ok(r)) {
        return Err(guest("A rating is between 1 and 5."));
    }
    let hash = create_entry(EntryTypes::Rating(Rating {
        proposal: input.proposal.clone(),
        membership,
        criterion: input.criterion,
        rating: input.rating,
    }))?;
    create_link(
        input.proposal,
        hash.clone(),
        LinkTypes::ProposalToRatings,
        (),
    )?;
    Ok(hash)
}
