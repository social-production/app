//! Which proposals passed (ADR 0033, 0040). For each proposal with a deciding Outcome (the
//! earliest citing every counted challenge), every visible vote cast before it is recounted
//! against the challenged population, leaving out voters found to be bots when the proposal
//! was made (ADR 0118); it passed only if that recount passes.

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{Proposal, ProposalKind, Vote, VoteChoice};
use sp_governance::Verdict;

use crate::calls;
use crate::count::count;
use crate::reads::{self, Found};
use crate::roll::Roll;

/// A passed proposal, with its recounted (eligible) votes.
pub struct Decision {
    pub at: Timestamp,
    pub proposal: Found<Proposal>,
    pub yes: u32,
    pub no: u32,
    /// Who wrote the Outcome: the vote (or organizer) that closed it.
    pub closed_by: AgentPubKey,
}

/// Whose votes count (not bots, ADR 0118), from the directory cell, asked once per (agent, proposal time).
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

/// The votes cast before `until` by people not found to be bots when the proposal was made:
/// the only ones a decision is applied on (ADR 0033, 0118).
pub fn eligible_votes(
    proposal: &Found<Proposal>,
    until: Timestamp,
    standing: &mut Standing,
) -> ExternResult<Vec<Found<Vote>>> {
    let mut eligible = Vec::new();
    for vote in reads::votes(&proposal.hash)?
        .into_iter()
        .filter(|v| v.at <= until)
    {
        if standing.full(&vote.author, proposal.at)? {
            eligible.push(vote);
        }
    }
    Ok(eligible)
}

/// Passed proposals whose recount passes too, in the order they were decided.
pub fn passed(standing: &mut Standing, roll: &Roll) -> ExternResult<Vec<Decision>> {
    let mut passed = Vec::new();
    let organized = project_integrity::organized()?;
    let ends = if organized {
        calls::organizer_ends()?
    } else {
        BTreeMap::new()
    };
    for proposal in reads::proposals()? {
        let count = count(&proposal, roll)?;
        let Some(outcome) = count.decided.as_ref().filter(|o| o.entry.passed) else {
            continue;
        };
        // Integrity accepts a vote-less pass only for an organizer's proposal (ADR 0061).
        // What a removed organizer decided afterwards doesn't count (ADR 0073).
        // Integrity accepts one for a personal service creator's request settings too
        // (ADR 0076).
        let saved = matches!(proposal.entry.kind, ProposalKind::RequestSettings { .. });
        if outcome.entry.votes.is_empty() && (organized || saved) {
            if organized && calls::lapsed(&ends, proposal.entry.organizer.as_ref(), proposal.at) {
                continue;
            }
            passed.push(Decision {
                at: outcome.at,
                closed_by: outcome.author.clone(),
                yes: 0,
                no: 0,
                proposal,
            });
            continue;
        }
        let eligible = eligible_votes(&proposal, outcome.at, standing)?;
        if count.tally(&eligible).verdict() == Verdict::Passed {
            let tally = |c: VoteChoice| {
                u32::try_from(eligible.iter().filter(|v| v.entry.choice == c).count()).unwrap_or(0)
            };
            passed.push(Decision {
                at: outcome.at,
                closed_by: outcome.author.clone(),
                yes: tally(VoteChoice::Yes),
                no: tally(VoteChoice::No),
                proposal,
            });
        }
    }
    passed.sort_by_key(|d| d.at);
    Ok(passed)
}
