//! The applied state: what passed decisions changed (ADR 0033). For each closed proposal every
//! visible vote cast before its first Outcome is recounted, keeping only voters with Full
//! standing from before the proposal; it applies only if that recount passes.

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{Applied, AppliedUpdate, Charter, Proposal, ProposalKind, VoteChoice};
use sp_governance::phases::{next_phase, Phase, Shape};
use sp_governance::plans::required_plan_kind;
use sp_governance::Verdict;

use crate::plans::{self, Approved};
use crate::reads::{self, Found};
use crate::{calls, write};

/// A passed proposal, with its recounted (eligible) votes.
pub struct Decision {
    pub at: Timestamp,
    pub proposal: Found<Proposal>,
    pub yes: u32,
    pub no: u32,
}

/// Full-standing answers from the directory cell, asked once per (agent, proposal time).
#[derive(Default)]
pub struct Standing(BTreeMap<(AgentPubKey, Timestamp), bool>);

impl Standing {
    pub fn full(&mut self, agent: &AgentPubKey, at: Timestamp) -> ExternResult<bool> {
        let key = (agent.clone(), at);
        if let Some(full) = self.0.get(&key) {
            return Ok(*full);
        }
        let full = calls::full_before(agent, at)?;
        self.0.insert(key, full);
        Ok(full)
    }
}

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
}

pub fn applied() -> ExternResult<Applied> {
    Ok(state(&mut Standing::default())?.applied)
}

pub fn state(standing: &mut Standing) -> ExternResult<State> {
    let decisions = passed(standing)?;
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

/// Passed proposals whose recount passes too, in the order they closed.
fn passed(standing: &mut Standing) -> ExternResult<Vec<Decision>> {
    let mut passed = Vec::new();
    for proposal in reads::proposals()? {
        let Some(outcome) = reads::outcome(&proposal.hash)? else {
            continue;
        };
        if !outcome.entry.passed {
            continue;
        }
        let mut eligible = Vec::new();
        for vote in reads::votes(&proposal.hash)?
            .into_iter()
            .filter(|v| v.at <= outcome.at)
        {
            if standing.full(&vote.author, proposal.at)? {
                eligible.push(vote);
            }
        }
        if write::tally_of(&proposal.entry, &eligible).verdict() == Verdict::Passed {
            let count = |c: VoteChoice| {
                u32::try_from(eligible.iter().filter(|v| v.entry.choice == c).count()).unwrap_or(0)
            };
            passed.push(Decision {
                at: outcome.at,
                yes: count(VoteChoice::Yes),
                no: count(VoteChoice::No),
                proposal,
            });
        }
    }
    passed.sort_by_key(|d| d.at);
    Ok(passed)
}

/// Apply passed decisions in the order they closed.
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

fn shape<'a>(mode: &'a str, subtype: &'a str) -> Shape<'a> {
    Shape {
        mode,
        subtype,
        include_access: false,
    }
}
