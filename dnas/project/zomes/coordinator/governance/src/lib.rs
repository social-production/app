//! Project governance API (docs/plans/M3-governance.md, ADR 0033): propose, vote, read
//! proposals with their tallies, and the applied state the project view is built from.

mod apply;
mod calls;
mod reads;
mod write;

use hdk::prelude::*;
use project_integrity::{Applied, ProposalKind, VoteChoice};
use sp_governance::Verdict;

#[derive(Serialize, Deserialize, Debug)]
pub struct VoteInput {
    pub proposal: ActionHash,
    pub choice: VoteChoice,
}

/// One proposal as the UI shows it (web's request cards: counts, quorum, the viewer's vote).
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
    /// "Open", "Passed" or "Rejected"; closed once an Outcome exists.
    pub status: Verdict,
    pub closed_at: Option<Timestamp>,
    pub my_vote: Option<VoteChoice>,
    /// The votes already decide it but no Outcome exists yet (a voter didn't see every vote
    /// when they cast theirs). Any member's app then calls `close`.
    pub settled: bool,
}

#[hdk_extern]
pub fn propose(kind: ProposalKind) -> ExternResult<ActionHash> {
    write::propose(kind)
}

#[hdk_extern]
pub fn vote(input: VoteInput) -> ExternResult<ActionHash> {
    write::vote(input.proposal, input.choice)
}

/// Write the Outcome for a settled proposal (anyone may; validators recompute it). Not named
/// `close`: hdk externs are unmangled symbols, and `close` would replace libc's on native builds.
#[hdk_extern]
pub fn close_proposal(proposal: ActionHash) -> ExternResult<Option<ActionHash>> {
    write::close(&proposal, None)
}

#[hdk_extern]
pub fn applied() -> ExternResult<Applied> {
    apply::applied()
}

/// Every proposal, newest first.
#[hdk_extern]
pub fn list_proposals() -> ExternResult<Vec<ProposalView>> {
    let me = agent_info()?.agent_initial_pubkey;
    let mut out = Vec::new();
    for p in reads::proposals()? {
        let votes = reads::votes(&p.hash)?;
        let tally = write::tally_of(&p.entry, &votes);
        let outcome = reads::outcome(&p.hash)?;
        let settled = outcome.is_none() && tally.verdict() != Verdict::Open;
        let count = |c: VoteChoice| {
            u32::try_from(votes.iter().filter(|v| v.entry.choice == c).count()).unwrap_or(0)
        };
        out.push(ProposalView {
            id: p.hash.clone(),
            author: p.author.clone(),
            kind: p.entry.kind.clone(),
            created_at: p.at,
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
            closed_at: outcome.map(|o| o.at),
            my_vote: votes
                .iter()
                .find(|v| v.author == me)
                .map(|v| v.entry.choice),
            settled,
        });
    }
    out.reverse();
    Ok(out)
}
