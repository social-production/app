//! Which proposals passed (ADR 0033, 0040). For each proposal with a deciding Outcome (the
//! earliest citing every counted challenge), every visible vote cast before it is recounted
//! against the challenged population, keeping only voters with Full standing from before the
//! proposal; it passed only if that recount passes.

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{Proposal, VoteChoice};
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

/// Passed proposals whose recount passes too, in the order they were decided.
pub fn passed(standing: &mut Standing, roll: &Roll) -> ExternResult<Vec<Decision>> {
    let mut passed = Vec::new();
    for proposal in reads::proposals()? {
        let count = count(&proposal, roll)?;
        let Some(outcome) = count.decided.as_ref().filter(|o| o.entry.passed) else {
            continue;
        };
        let mut eligible = Vec::new();
        for vote in reads::votes(&proposal.hash)?
            .into_iter()
            .filter(|v| v.at <= outcome.at)
        {
            if standing.full(&vote.author, proposal.at)? {
                eligible.push(vote);
            }
        }
        if count.tally(&eligible).verdict() == Verdict::Passed {
            let tally = |c: VoteChoice| {
                u32::try_from(eligible.iter().filter(|v| v.entry.choice == c).count()).unwrap_or(0)
            };
            passed.push(Decision {
                at: outcome.at,
                yes: tally(VoteChoice::Yes),
                no: tally(VoteChoice::No),
                proposal,
            });
        }
    }
    passed.sort_by_key(|d| d.at);
    Ok(passed)
}
