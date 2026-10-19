//! Where a proposal's count stands once challenges are taken into account (ADR 0040): the
//! population N, the Outcome that decides it (if any), and who is still left out.

use std::collections::BTreeSet;

use hdk::prelude::*;
use project_integrity::{Outcome, Proposal, Vote, VoteChoice};
use sp_governance::challenge::{added, deciding, in_time, window_end};
use sp_governance::{tally, Choice, Tally, Threshold};

use crate::reads::{self, Found};
use crate::roll::Roll;

pub struct Count {
    /// The snapshot plus the members counted challenges add.
    pub population: u32,
    /// Members counted challenges added.
    pub added: u32,
    /// Challenges that count: in time, naming only memberships live when the proposal was made.
    pub challenges: Vec<ActionHash>,
    /// The earliest Outcome citing every counted challenge. `None`: open (or reopened).
    pub decided: Option<Found<Outcome>>,
    /// When challenges stop counting (the window after the first Outcome), once there is one.
    pub window_end: Option<Timestamp>,
    /// Memberships from before the proposal that neither the snapshot nor a challenge counts.
    pub missing: Vec<ActionHash>,
}

impl Count {
    /// The tally of `votes` against this count's population.
    pub fn tally(&self, votes: &[Found<Vote>]) -> Tally {
        let choices: Vec<Choice> = votes
            .iter()
            .filter_map(|v| match v.entry.choice {
                VoteChoice::Yes => Some(Choice::Yes),
                VoteChoice::No => Some(Choice::No),
                VoteChoice::Neutral => None,
            })
            .collect();
        tally(&choices, self.population, Threshold::DEFAULT)
    }

    /// Whether a challenge written now would still count.
    pub fn open_to_challenge(&self, now: Timestamp) -> bool {
        self.window_end.is_none_or(|end| now <= end)
    }
}

pub fn count(p: &Found<Proposal>, roll: &Roll) -> ExternResult<Count> {
    let outcomes = reads::outcomes(&p.hash)?;
    let first = outcomes.first().map(|o| o.at.as_micros());
    let mut snapshot = BTreeSet::new();
    for hash in &p.entry.members {
        snapshot.extend(roll.agent(hash)?);
    }
    let mut counted = BTreeSet::new();
    let mut challenged = Vec::new();
    for c in reads::challenges(&p.hash)? {
        if !in_time(first, c.at.as_micros()) {
            continue;
        }
        // A challenge naming someone who had already left is void as a whole, so the Outcome
        // citing the counted ones gets the same population from integrity.
        let named: Option<Vec<AgentPubKey>> = c
            .entry
            .omitted
            .iter()
            .map(|h| roll.member_at(h, p.at).cloned())
            .collect();
        if let Some(named) = named {
            counted.insert(c.hash);
            challenged.extend(named);
        }
    }
    let added = added(&snapshot, challenged);
    let cited: Vec<(i64, BTreeSet<ActionHash>)> = outcomes
        .iter()
        .map(|o| {
            (
                o.at.as_micros(),
                o.entry.challenges.iter().cloned().collect(),
            )
        })
        .collect();
    let decided = deciding(&cited, &counted).and_then(|i| outcomes.into_iter().nth(i));
    let missing = roll
        .members_at(p.at)
        .into_iter()
        .filter(|(agent, _)| !snapshot.contains(agent) && !added.contains(agent))
        .map(|(_, hash)| hash)
        .collect();
    let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Ok(Count {
        population: to_u32(p.entry.members.len() + added.len()),
        added: to_u32(added.len()),
        challenges: counted.into_iter().collect(),
        decided,
        window_end: first.map(|f| Timestamp::from_micros(window_end(f))),
        missing,
    })
}
