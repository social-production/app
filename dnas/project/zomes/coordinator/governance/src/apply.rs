//! The applied state: what passed decisions changed (ADR 0033). For each closed proposal every
//! visible vote cast before its first Outcome is recounted, keeping only voters with Full
//! standing from before the proposal; it applies only if that recount passes.

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{Applied, AppliedUpdate, ProposalKind};
use sp_governance::phases::{next_phase, Phase, Shape};
use sp_governance::Verdict;

use crate::reads::{self, Found};
use crate::{calls, write};

struct Decision {
    at: Timestamp,
    proposal: Found<project_integrity::Proposal>,
}

pub fn applied() -> ExternResult<Applied> {
    let mut standing: BTreeMap<(AgentPubKey, Timestamp), bool> = BTreeMap::new();
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
            let key = (vote.author.clone(), proposal.at);
            if !standing.contains_key(&key) {
                let full = calls::full_before(&vote.author, proposal.at)?;
                standing.insert(key.clone(), full);
            }
            if standing.get(&key) == Some(&true) {
                eligible.push(vote);
            }
        }
        if write::tally_of(&proposal.entry, &eligible).verdict() == Verdict::Passed {
            passed.push(Decision {
                at: outcome.at,
                proposal,
            });
        }
    }
    passed.sort_by_key(|d| d.at);
    fold(passed)
}

/// Apply passed decisions in the order they closed.
fn fold(decisions: Vec<Decision>) -> ExternResult<Applied> {
    let charter = calls::base_charter()?;
    let shape = charter
        .as_ref()
        .map(|c| (c.mode.clone(), c.subtype.clone()));
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
            ProposalKind::Phase { to } => {
                // Only a move to the next phase from where the project is applies.
                let Some((mode, subtype)) = &shape else {
                    continue;
                };
                let shape = Shape {
                    mode,
                    subtype,
                    include_access: false,
                };
                if let Some(next) = next_phase(shape, phase).filter(|n| n.id() == to) {
                    phase = next;
                    out.phase = Some(next.id().to_string());
                }
            }
        }
    }
    out.updates.reverse(); // newest first, as web lists them
    Ok(out)
}
