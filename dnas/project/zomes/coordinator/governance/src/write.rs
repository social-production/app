//! Proposing, voting, closing and challenging. Each fails early with a readable reason where
//! integrity (or the apply-time rules, ADR 0033) would not accept the result.

use hdk::prelude::*;
use project_integrity::{
    Challenge, EntryTypes, LinkTypes, Outcome, Proposal, ProposalKind, Vote, VoteChoice,
};
use sp_governance::Verdict;

use crate::apply::{self, Standing};
use crate::count::count;
use crate::reads::{self, Found};
use crate::roll::Roll;
use crate::{calls, link_write, moves, plan_write};

pub fn guest(message: impl Into<String>) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(message.into()))
}

/// Help requests, threads and posts have no lifecycle, plans or links (ADR 0067, 0070).
fn takes_proposals() -> ExternResult<()> {
    match calls::base_charter()?.map(|c| c.mode) {
        Some(m) if m == sp_help::MODE => Err(guest("Help requests don't take proposals.")),
        Some(m) if sp_discussion::is_discussion(&m) => {
            Err(guest("Threads and posts don't take proposals."))
        }
        _ => Ok(()),
    }
}

pub fn propose(kind: ProposalKind) -> ExternResult<ActionHash> {
    takes_proposals()?;
    if let ProposalKind::Link(link) = kind {
        return link_write::propose(link);
    }
    let membership =
        calls::my_membership()?.ok_or_else(|| guest("Join the project to propose changes."))?;
    let charter =
        calls::base_charter()?.ok_or_else(|| guest("This project has no charter yet."))?;
    accepts(&charter.mode, &kind)?;
    // Organizer-controlled events (ADR 0061): only organizers propose, and it applies at once.
    let organized = project_integrity::organized()?;
    let organizer =
        if organized {
            calls::my_organizer_role()?.ok_or_else(|| {
            guest("Only the creator or co-organizers can manage this organizer-controlled event.")
        })?
        .grant()
        } else {
            None
        };
    let roll = Roll::load()?;
    let kind = match kind {
        ProposalKind::Phase { to, reason, .. } => {
            if reason.trim().is_empty() {
                return Err(guest("Add a reason before submitting this phase change."));
            }
            let state = apply::state(&mut Standing::default(), &roll)?;
            let plan = moves::check_phase_move(&state, &to)?;
            ProposalKind::Phase { to, plan, reason }
        }
        ProposalKind::Plan(plan) => {
            let state = apply::state(&mut Standing::default(), &roll)?;
            ProposalKind::Plan(plan_write::prepare(&state, plan)?)
        }
        ProposalKind::Return { to, reason } => {
            let state = apply::state(&mut Standing::default(), &roll)?;
            moves::check_return(&state, &to, &reason, &roll)?;
            ProposalKind::Return { to, reason }
        }
        other => other,
    };
    // The proposer's own Yes comes with the proposal (web's proposer vote). Plans are assessed
    // first and returns are opened without one, as on web (`revert.py`); their authors vote
    // like everyone else.
    let proposer_votes = !matches!(kind, ProposalKind::Plan(_) | ProposalKind::Return { .. });
    let members = calls::member_snapshot()?;
    let hash = create_entry(EntryTypes::Proposal(Proposal {
        space: space_scope::current()?,
        kind,
        membership: Some(membership),
        members,
        organizer,
    }))?;
    create_link(space_scope::base()?, hash.clone(), LinkTypes::Proposals, ())?;
    if organized {
        decide_now(&hash)?;
    } else if proposer_votes {
        vote(hash.clone(), VoteChoice::Yes)?;
    }
    Ok(hash)
}

/// Personal services take no proposals; channels and communities take removals but keep
/// their handle (ADR 0068).
fn accepts(mode: &str, kind: &ProposalKind) -> ExternResult<()> {
    if mode == "personal-service" {
        return Err(guest("Personal services don't take proposals."));
    }
    project_integrity::check_scope_kind(mode, kind).map_err(guest)
}

pub fn vote(proposal: ActionHash, choice: VoteChoice) -> ExternResult<ActionHash> {
    let membership = calls::my_membership()?.ok_or_else(|| guest("Join the project to vote."))?;
    let me = agent_info()?.agent_initial_pubkey;
    if reads::votes(&proposal)?.iter().any(|v| v.author == me) || voted_locally(&proposal)? {
        return Err(guest("You've already voted on this."));
    }
    let found = reads::proposal(&proposal)?
        .ok_or_else(|| guest("This proposal hasn't reached your node yet."))?;
    if count(&found, &Roll::load()?)?.decided.is_some() {
        return Err(guest("This vote has closed."));
    }
    let hash = create_entry(EntryTypes::Vote(Vote {
        proposal: proposal.clone(),
        membership,
        choice,
    }))?;
    create_link(
        proposal.clone(),
        hash.clone(),
        LinkTypes::ProposalToVotes,
        (),
    )?;
    close(&proposal, reads::found::<Vote>(hash.clone())?)?;
    Ok(hash)
}

/// An organizer's proposal passes without votes (ADR 0061).
pub(crate) fn decide_now(proposal: &ActionHash) -> ExternResult<ActionHash> {
    let outcome = Outcome {
        proposal: proposal.clone(),
        votes: Vec::new(),
        passed: true,
        challenges: Vec::new(),
    };
    let hash = create_entry(EntryTypes::Outcome(outcome))?;
    create_link(
        proposal.clone(),
        hash.clone(),
        LinkTypes::ProposalToOutcomes,
        (),
    )?;
    Ok(hash)
}

/// Own chain: a vote just written may not be visible on the network yet.
fn voted_locally(proposal: &ActionHash) -> ExternResult<bool> {
    let votes = EntryType::App(project_integrity::UnitEntryTypes::Vote.try_into()?);
    let mine = query(
        ChainQueryFilter::new()
            .entry_type(votes)
            .include_entries(true),
    )?;
    Ok(mine
        .iter()
        .filter_map(|r| r.entry().to_app_option::<Vote>().ok().flatten())
        .any(|v| &v.proposal == proposal))
}

/// Write the Outcome once the result is settled; returns the deciding one when it exists.
/// `fresh` is a vote just written here, which the network may not show yet. The Outcome cites
/// the counted challenges, so integrity counts the same population (ADR 0040).
pub fn close(
    proposal_hash: &ActionHash,
    fresh: Option<Found<Vote>>,
) -> ExternResult<Option<ActionHash>> {
    let Some(proposal) = reads::proposal(proposal_hash)? else {
        return Ok(None);
    };
    let count = count(&proposal, &Roll::load()?)?;
    if let Some(decided) = count.decided {
        return Ok(Some(decided.hash));
    }
    let mut votes = reads::votes(proposal_hash)?;
    if let Some(v) = fresh.filter(|f| votes.iter().all(|v| v.author != f.author)) {
        votes.push(v);
    }
    let passed = match count.tally(&votes).verdict() {
        Verdict::Open => return Ok(None),
        Verdict::Passed => true,
        Verdict::Rejected => false,
    };
    let outcome = Outcome {
        proposal: proposal_hash.clone(),
        votes: votes.iter().map(|v| v.hash.clone()).collect(),
        passed,
        challenges: count.challenges,
    };
    let hash = create_entry(EntryTypes::Outcome(outcome))?;
    create_link(
        proposal_hash.clone(),
        hash.clone(),
        LinkTypes::ProposalToOutcomes,
        (),
    )?;
    Ok(Some(hash))
}

/// Cite every member the proposal's count leaves out (docs/GOVERNANCE.md, ADR 0040). The
/// proposal reopens with the larger population; it closes again once its votes decide it.
pub fn challenge(proposal: ActionHash) -> ExternResult<ActionHash> {
    let membership = calls::my_membership()?
        .ok_or_else(|| guest("Join the project to challenge a decision."))?;
    let found = reads::proposal(&proposal)?
        .ok_or_else(|| guest("This proposal hasn't reached your node yet."))?;
    let count = count(&found, &Roll::load()?)?;
    if !count.open_to_challenge(sys_time()?) {
        return Err(guest("The time to challenge this decision has passed."));
    }
    if count.missing.is_empty() {
        return Err(guest(
            "Everyone who was a member when this was proposed is already counted.",
        ));
    }
    let hash = create_entry(EntryTypes::Challenge(Challenge {
        proposal: proposal.clone(),
        membership,
        omitted: count.missing,
    }))?;
    create_link(proposal, hash.clone(), LinkTypes::ProposalToChallenges, ())?;
    Ok(hash)
}
