//! What the UI shows of a proposal's votes (web's request cards: counts, quorum, the viewer's
//! vote), shared by the proposal list and the plan board, plus its challenge state (ADR 0040).

use hdk::prelude::*;
use project_integrity::{Proposal, ProposalKind, VoteChoice};
use sp_governance::Verdict;

use crate::count::count;
use crate::reads::{self, Found};
use crate::roll::Roll;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TallyView {
    pub yes: u32,
    pub no: u32,
    pub neutral: u32,
    /// The snapshot plus the members challenges added.
    pub population: u32,
    pub required: u32,
    /// "Open", "Passed" or "Rejected"; closed once a deciding Outcome exists.
    pub status: Verdict,
    pub closed_at: Option<Timestamp>,
    pub my_vote: Option<VoteChoice>,
    /// The votes already decide it but no deciding Outcome exists yet (a voter didn't see every
    /// vote, or a challenge reopened it). Any member's app then calls `close_proposal`.
    pub settled: bool,
    /// Members challenges added to the count.
    pub added: u32,
    /// Members from before the proposal the count still leaves out, as this node sees them.
    pub missing: u32,
    /// Challenges count until then (48 h after the first Outcome); `None` while undecided.
    pub challenge_until: Option<Timestamp>,
    /// Someone is left out and a challenge would still count.
    pub can_challenge: bool,
}

pub fn tally_view(p: &Found<Proposal>, me: &AgentPubKey, roll: &Roll) -> ExternResult<TallyView> {
    let votes = reads::votes(&p.hash)?;
    let count = count(p, roll)?;
    let tally = count.tally(&votes);
    let tallied = |c: VoteChoice| {
        u32::try_from(votes.iter().filter(|v| v.entry.choice == c).count()).unwrap_or(0)
    };
    let missing = u32::try_from(count.missing.len()).unwrap_or(u32::MAX);
    Ok(TallyView {
        yes: tallied(VoteChoice::Yes),
        no: tallied(VoteChoice::No),
        neutral: tallied(VoteChoice::Neutral),
        population: tally.population,
        required: tally.required,
        status: count.decided.as_ref().map_or(Verdict::Open, |o| {
            if o.entry.passed {
                Verdict::Passed
            } else {
                Verdict::Rejected
            }
        }),
        settled: count.decided.is_none() && tally.verdict() != Verdict::Open,
        closed_at: count.decided.as_ref().map(|o| o.at),
        my_vote: votes
            .iter()
            .find(|v| &v.author == me)
            .map(|v| v.entry.choice),
        added: count.added,
        missing,
        challenge_until: count.window_end,
        can_challenge: missing > 0 && count.open_to_challenge(sys_time()?),
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
    pub added: u32,
    pub missing: u32,
    pub challenge_until: Option<Timestamp>,
    pub can_challenge: bool,
}

/// Every proposal except plans and links (they have their own boards), newest first.
pub fn list_proposals() -> ExternResult<Vec<ProposalView>> {
    let me = agent_info()?.agent_initial_pubkey;
    let roll = Roll::load()?;
    let mut out = Vec::new();
    for p in reads::proposals()? {
        if matches!(p.entry.kind, ProposalKind::Plan(_) | ProposalKind::Link(_)) {
            continue;
        }
        let t = tally_view(&p, &me, &roll)?;
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
            added: t.added,
            missing: t.missing,
            challenge_until: t.challenge_until,
            can_challenge: t.can_challenge,
        });
    }
    out.reverse();
    Ok(out)
}
