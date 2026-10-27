//! Checks before proposing a phase change, with web's refusals: moving on (ADR 0033, 0039)
//! and returning to an earlier phase (web-backend `projects/phases/revert.py`, ADR 0040).

use hdk::prelude::*;
use project_integrity::ProposalKind;
use sp_governance::phases::{next_phase, signal_gate_met, Phase};
use sp_governance::plans::{required_plan_kind, Leader};
use sp_governance::returns::can_return;

use crate::apply::State;
use crate::count::count;
use crate::roll::Roll;
use crate::write::guest;
use crate::{calls, plans, reads};

/// A phase move must be the next phase; leaving Proposal needs ≥ 66% demand (ADR 0033), and
/// leaving a plan phase needs an approved leading plan, which the move cites (ADR 0039).
pub fn check_phase_move(state: &State, to: &str) -> ExternResult<Option<ActionHash>> {
    let shape = state.shape();
    let current = state.phase();
    if next_phase(shape, current).map(Phase::id) != Some(to) {
        return Err(guest("A project can only move to its next phase."));
    }
    // Organizer-controlled events skip signals (ADR 0061).
    if current == Phase::P1 && !project_integrity::organized()? {
        let s = calls::signal_counts()?;
        if !signal_gate_met(s.demand, s.opposition) {
            return Err(guest(
                "Planning opens once support is above 66% of all signals.",
            ));
        }
    }
    let Some(kind) = required_plan_kind(shape, current) else {
        return Ok(None);
    };
    match plans::leader(&state.approved, kind) {
        Leader::One(plan) => Ok(Some(plan)),
        _ => Err(guest(
            "An approved plan is required before advancing this project phase.",
        )),
    }
}

/// web's revert request: an earlier planning phase the project shows, a reason, and no other
/// phase vote open (web-backend refuses a second open return; web's menu locks while any
/// phase vote is open).
pub fn check_return(state: &State, to: &str, reason: &str, roll: &Roll) -> ExternResult<()> {
    let target = Phase::parse(to);
    if !target.is_some_and(|t| can_return(state.shape(), state.phase(), t)) {
        return Err(guest(
            "A project can only return to an earlier planning phase.",
        ));
    }
    if target == Some(Phase::P1) && project_integrity::organized()? {
        return Err(guest(
            "Organizer-controlled events can't return to Proposal.",
        ));
    }
    if reason.trim().is_empty() {
        return Err(guest("Add a reason before submitting this return request."));
    }
    if phase_vote_open(roll)? {
        return Err(guest(
            "A vote is already open — approve or reject it first.",
        ));
    }
    Ok(())
}

/// Whether a move or return is still being decided.
fn phase_vote_open(roll: &Roll) -> ExternResult<bool> {
    for p in reads::proposals()? {
        let phase_change = matches!(
            p.entry.kind,
            ProposalKind::Phase { .. } | ProposalKind::Return { .. }
        );
        if phase_change && count(&p, roll)?.decided.is_none() {
            return Ok(true);
        }
    }
    Ok(false)
}
