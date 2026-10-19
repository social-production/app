//! What the UI shows of a proposal's votes (web's request cards: counts, quorum, the viewer's
//! vote), shared by the proposal list and the plan board.

use hdk::prelude::*;
use project_integrity::{Proposal, ProposalKind, VoteChoice};
use sp_governance::Verdict;

use crate::reads::{self, Found};
use crate::write;

#[derive(Serialize, Deserialize, Debug)]
pub struct TallyView {
    pub yes: u32,
    pub no: u32,
    pub neutral: u32,
    pub population: u32,
    pub required: u32,
    /// "Open", "Passed" or "Rejected"; closed once an Outcome exists.
    pub status: Verdict,
    pub closed_at: Option<Timestamp>,
    pub my_vote: Option<VoteChoice>,
    /// The votes already decide it but no Outcome exists yet (a voter didn't see every vote
    /// when they cast theirs). Any member's app then calls `close`.
    pub settled: bool,
}

pub fn tally_view(p: &Found<Proposal>, me: &AgentPubKey) -> ExternResult<TallyView> {
    let votes = reads::votes(&p.hash)?;
    let tally = write::tally_of(&p.entry, &votes);
    let outcome = reads::outcome(&p.hash)?;
    let count = |c: VoteChoice| {
        u32::try_from(votes.iter().filter(|v| v.entry.choice == c).count()).unwrap_or(0)
    };
    Ok(TallyView {
        yes: count(VoteChoice::Yes),
        no: count(VoteChoice::No),
        neutral: count(VoteChoice::Neutral),
        population: tally.population,
        required: tally.required,
        status: outcome.as_ref().map_or(Verdict::Open, |o| {
            if o.entry.passed {
                Verdict::Passed
            } else {
                Verdict::Rejected
            }
        }),
        settled: outcome.is_none() && tally.verdict() != Verdict::Open,
        closed_at: outcome.map(|o| o.at),
        my_vote: votes
            .iter()
            .find(|v| &v.author == me)
            .map(|v| v.entry.choice),
    })
}

/// One proposal as the vote dock shows it.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProposalView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub kind: ProposalKind,
    pub created_at: Timestamp,
    pub yes: u32,
    pub no: u32,
    pub neutral: u32,
    pub population: u32,
    pub required: u32,
    pub status: Verdict,
    pub closed_at: Option<Timestamp>,
    pub my_vote: Option<VoteChoice>,
    pub settled: bool,
}

/// Every proposal except plans (they have their own board), newest first.
pub fn list_proposals() -> ExternResult<Vec<ProposalView>> {
    let me = agent_info()?.agent_initial_pubkey;
    let mut out = Vec::new();
    for p in reads::proposals()? {
        if matches!(p.entry.kind, ProposalKind::Plan(_)) {
            continue;
        }
        let t = tally_view(&p, &me)?;
        out.push(ProposalView {
            id: p.hash,
            author: p.author,
            kind: p.entry.kind,
            created_at: p.at,
            yes: t.yes,
            no: t.no,
            neutral: t.neutral,
            population: t.population,
            required: t.required,
            status: t.status,
            closed_at: t.closed_at,
            my_vote: t.my_vote,
            settled: t.settled,
        });
    }
    out.reverse();
    Ok(out)
}
