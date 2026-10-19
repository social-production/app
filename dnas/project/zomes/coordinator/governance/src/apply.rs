//! The applied state: what passed decisions changed (ADR 0033), folded in the order they were
//! decided (`decide`). Returns go back to an earlier planning phase (ADR 0040).

use hdk::prelude::*;
use project_integrity::{Applied, AppliedUpdate, Charter, ProposalKind};
use sp_governance::phases::{next_phase, Phase, Shape};
use sp_governance::plans::required_plan_kind;
use sp_governance::returns::{can_return, reopens_subtype};

pub use crate::decide::{Decision, Standing};
use crate::plans::{self, Approved};
use crate::roll::Roll;
use crate::{calls, decide};

/// Everything the decisions add up to, for the views and for the next proposal's checks.
pub struct State {
    pub charter: Option<Charter>,
    pub applied: Applied,
    pub approved: Vec<Approved>,
}

impl State {
    pub fn phase(&self) -> Phase {
        self.applied
            .phase
            .as_deref()
            .and_then(Phase::parse)
            .unwrap_or(Phase::P1)
    }

    /// The project's subtype now: chosen by a plan, else the charter's.
    pub fn subtype(&self) -> &str {
        self.applied
            .subtype
            .as_deref()
            .or(self.charter.as_ref().map(|c| c.subtype.as_str()))
            .unwrap_or("standard")
    }

    pub fn mode(&self) -> &str {
        self.charter
            .as_ref()
            .map_or("productive", |c| c.mode.as_str())
    }

    pub fn shape(&self) -> Shape<'_> {
        shape(self.mode(), self.subtype())
    }
}

pub fn applied() -> ExternResult<Applied> {
    Ok(state(&mut Standing::default(), &Roll::load()?)?.applied)
}

pub fn state(standing: &mut Standing, roll: &Roll) -> ExternResult<State> {
    let decisions = decide::passed(standing, roll)?;
    let charter = calls::base_charter()?;
    let mode = charter.as_ref().map_or("", |c| c.mode.as_str());
    let approved = plans::approved(&decisions, mode, standing)?;
    let mut applied = fold(decisions, charter.as_ref(), &approved);
    if applied.subtype.is_none() {
        // While the production/operations plan is still being chosen, its leader sets the
        // subtype live, as on web; leaving the phase freezes the cited plan's.
        applied.subtype = plans::live_subtype(&approved);
    }
    Ok(State {
        charter,
        applied,
        approved,
    })
}

/// Apply passed decisions in the order they were decided.
fn fold(decisions: Vec<Decision>, charter: Option<&Charter>, approved: &[Approved]) -> Applied {
    let mut out = Applied::default();
    let mut phase = Phase::P1;
    for d in decisions {
        out.last_decision_at = Some(d.at);
        match d.proposal.entry.kind {
            ProposalKind::Edit {
                title,
                description,
                location,
            } => {
                out.title = title.or(out.title);
                out.description = description.or(out.description);
                out.location = location.or(out.location);
            }
            ProposalKind::Update { body } => out.updates.push(AppliedUpdate {
                proposal: d.proposal.hash,
                author: d.proposal.author,
                body,
                at: d.at,
            }),
            ProposalKind::Phase { to, plan } => {
                let Some(c) = charter else { continue };
                if let Some((next, subtype)) = phase_move(c, &out, phase, &to, plan, d.at, approved)
                {
                    phase = next;
                    out.phase = Some(next.id().to_string());
                    out.subtype = subtype.or(out.subtype);
                }
            }
            ProposalKind::Return { to, .. } => {
                let Some(c) = charter else { continue };
                if let Some(back) = phase_return(c, &out, phase, &to) {
                    phase = back;
                    out.phase = Some(back.id().to_string());
                    if reopens_subtype(back) {
                        out.subtype = None;
                    }
                }
            }
            // An approved plan changes nothing by itself; phase moves cite it.
            ProposalKind::Plan(_) => {}
        }
    }
    out.updates.reverse(); // newest first, as web lists them
    out
}

/// A phase move applies only as the next step from where the project is; leaving a plan
/// phase also needs the cited plan approved (before this move closed) for that phase
/// (web's `_ensure_project_phase_plan_gate`, ADR 0039). Returns the new phase and the subtype
/// the cited plan chose.
fn phase_move(
    charter: &Charter,
    so_far: &Applied,
    phase: Phase,
    to: &str,
    plan: Option<ActionHash>,
    at: Timestamp,
    approved: &[Approved],
) -> Option<(Phase, Option<String>)> {
    let current = so_far
        .subtype
        .clone()
        .unwrap_or_else(|| charter.subtype.clone());
    let mut chosen = None;
    if let Some(kind) = required_plan_kind(shape(&charter.mode, &current), phase) {
        let cited = approved
            .iter()
            .find(|a| Some(&a.hash) == plan.as_ref() && a.kind == kind && a.at < at)?;
        if kind.sets_subtype() {
            chosen.clone_from(&cited.subtype);
        }
    }
    let subtype = chosen.clone().unwrap_or_else(|| current.clone());
    next_phase(shape(&charter.mode, &subtype), phase)
        .filter(|n| n.id() == to)
        .map(|n| (n, chosen))
}

/// A return applies only to an earlier planning phase the project shows from where it is now
/// (web's `revertablePhaseIds`), so a return decided after another move may no longer apply.
fn phase_return(charter: &Charter, so_far: &Applied, phase: Phase, to: &str) -> Option<Phase> {
    let subtype = so_far.subtype.as_deref().unwrap_or(&charter.subtype);
    Phase::parse(to).filter(|t| can_return(shape(&charter.mode, subtype), phase, *t))
}

fn shape<'a>(mode: &'a str, subtype: &'a str) -> Shape<'a> {
    Shape {
        mode,
        subtype,
        include_access: false,
    }
}
