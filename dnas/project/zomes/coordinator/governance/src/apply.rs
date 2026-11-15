//! The applied state: what passed decisions changed (ADR 0033), folded in the order they were
//! decided (`decide`). Returns go back to an earlier planning phase (ADR 0040).

use hdk::prelude::*;
use project_integrity::{Applied, AppliedRequests, AppliedUpdate, Charter, ProposalKind};
use sp_governance::phases::{next_phase, Phase, Shape, PROPOSAL};
use sp_governance::plans::required_plan_kinds;
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
    /// Passed software proposals and when they passed (replayed by `software`, ADR 0086).
    pub software: Vec<(ActionHash, Timestamp)>,
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

    /// The space's type now: what its leading plan made a proposal (ADR 0059), else the
    /// charter's. A proposal still choosing its plan is a `proposal`.
    pub fn mode(&self) -> &str {
        self.applied.mode.as_deref().unwrap_or(self.born())
    }

    /// The mode the space was founded with; a `proposal` stays one here whatever it became.
    pub fn born(&self) -> &str {
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
    let software = decisions
        .iter()
        .filter(|d| matches!(d.proposal.entry.kind, ProposalKind::Software(_)))
        .map(|d| (d.proposal.hash.clone(), d.at))
        .collect();
    let mut applied = fold(decisions, charter.as_ref(), &approved);
    if applied.subtype.is_none() {
        // While the production/operations plan is still being chosen, its leader sets the
        // subtype live, as on web; leaving the phase freezes the cited plan's.
        applied.subtype = plans::live_subtype(&approved, mode);
    }
    if mode == PROPOSAL && applied.mode.is_none() {
        applied.leading_mode = plans::first_leader(&approved, mode)
            .and_then(|a| a.kind.makes())
            .map(str::to_string);
    }
    Ok(State {
        charter,
        applied,
        approved,
        software,
    })
}

/// A passed edit: the fields it changes; a new location brings its place, or none (ADR 0078).
fn edit(out: &mut Applied, kind: ProposalKind) {
    if let ProposalKind::Edit {
        title,
        description,
        location,
        place,
    } = kind
    {
        out.title = title.or(out.title.take());
        out.description = description.or(out.description.take());
        if location.is_some() {
            out.location = location;
            out.place = place.map(|p| *p);
        }
    }
}

/// Apply passed decisions in the order they were decided.
fn fold(decisions: Vec<Decision>, charter: Option<&Charter>, approved: &[Approved]) -> Applied {
    let mut out = Applied::default();
    let mut phase = Phase::P1;
    for d in decisions {
        out.last_decision_at = Some(d.at);
        match d.proposal.entry.kind {
            kind @ ProposalKind::Edit { .. } => edit(&mut out, kind),
            ProposalKind::Update { body } => out.updates.push(AppliedUpdate {
                proposal: d.proposal.hash,
                author: d.proposal.author,
                body,
                at: d.at,
            }),
            ProposalKind::Phase { to, plan, .. } => {
                let Some(c) = charter else { continue };
                if let Some(moved) = phase_move(c, &out, phase, &to, plan, d.at, approved) {
                    phase = moved.to;
                    out.phase = Some(moved.to.id().to_string());
                    out.subtype = moved.subtype.or(out.subtype);
                    out.mode = moved.mode.or(out.mode);
                    if let Some(mode) = moved.requests {
                        // The cited operations plan switched requests on (ADR 0059); a
                        // later `RequestSettings` vote changes them.
                        out.requests = Some(AppliedRequests {
                            enabled: true,
                            off_schedule: mode == "both",
                            mode,
                            at: d.at,
                        });
                    }
                }
            }
            ProposalKind::Return { to, .. } => {
                let Some(c) = charter else { continue };
                if let Some(back) = phase_return(c, &out, phase, &to) {
                    phase = back;
                    out.phase = Some(back.id().to_string());
                    if reopens_subtype(back) {
                        // Back in the first plan phase or before it: the subtype, and a
                        // proposal's type, are open again (ADR 0040, 0059).
                        out.subtype = None;
                        out.mode = None;
                        out.requests = None;
                    }
                }
            }
            // An approved plan changes nothing by itself; phase moves cite it. A link needs
            // both projects' approval, which the Links tab pairs (ADR 0042). Software
            // decisions are replayed on their own (`software`, ADR 0086).
            ProposalKind::Plan(_) | ProposalKind::Link(_) | ProposalKind::Software(_) => {}
            ProposalKind::RemoveMember { agent } => out.removed.push((agent, d.at)),
            ProposalKind::RequestSettings {
                enabled,
                mode,
                off_schedule,
                ..
            } => {
                out.requests = Some(AppliedRequests {
                    enabled,
                    mode,
                    off_schedule,
                    at: d.at,
                });
            }
        }
    }
    out.updates.reverse(); // newest first, as web lists them
    out
}

/// What a passed phase move changed.
struct Moved {
    to: Phase,
    /// The subtype the cited plan chose, if it chooses one.
    subtype: Option<String>,
    /// The type the cited plan gave a proposal (ADR 0059).
    mode: Option<String>,
    /// The request mode the cited operations plan switched on (ADR 0059).
    requests: Option<String>,
}

/// A phase move applies only as the next step from where the project is; leaving a plan
/// phase also needs the cited plan approved (before this move closed) for that phase
/// (web's `_ensure_project_phase_plan_gate`, ADR 0039). A proposal leaves its Plan phase as
/// the type its cited plan makes it, into that type's next phase (ADR 0059).
fn phase_move(
    charter: &Charter,
    so_far: &Applied,
    phase: Phase,
    to: &str,
    plan: Option<ActionHash>,
    at: Timestamp,
    approved: &[Approved],
) -> Option<Moved> {
    let current = so_far
        .subtype
        .clone()
        .unwrap_or_else(|| charter.subtype.clone());
    let mode = so_far.mode.as_deref().unwrap_or(&charter.mode);
    let kinds = required_plan_kinds(shape(mode, &current), phase);
    let mut chosen = None;
    let mut made = None;
    let mut requests = None;
    if !kinds.is_empty() {
        let cited = approved
            .iter()
            .find(|a| Some(&a.hash) == plan.as_ref() && kinds.contains(&a.kind) && a.at < at)?;
        if cited.kind.sets_subtype() {
            chosen.clone_from(&cited.subtype);
        }
        if mode == PROPOSAL {
            made = cited.kind.makes().map(str::to_string);
        }
        requests.clone_from(&cited.requests);
    }
    let subtype = chosen.clone().unwrap_or_else(|| current.clone());
    let leaving_as = made.as_deref().unwrap_or(mode);
    next_phase(shape(leaving_as, &subtype), phase)
        .filter(|n| n.id() == to)
        .map(|to| Moved {
            to,
            subtype: chosen,
            mode: made,
            requests,
        })
}

/// A return applies only to an earlier planning phase the project shows from where it is now
/// (web's `revertablePhaseIds`), so a return decided after another move may no longer apply.
fn phase_return(charter: &Charter, so_far: &Applied, phase: Phase, to: &str) -> Option<Phase> {
    let subtype = so_far.subtype.as_deref().unwrap_or(&charter.subtype);
    let mode = so_far.mode.as_deref().unwrap_or(&charter.mode);
    Phase::parse(to).filter(|t| can_return(shape(mode, subtype), phase, *t))
}

fn shape<'a>(mode: &'a str, subtype: &'a str) -> Shape<'a> {
    Shape {
        mode,
        subtype,
        include_access: false,
    }
}
