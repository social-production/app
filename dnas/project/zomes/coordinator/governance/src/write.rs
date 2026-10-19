//! Proposing, voting and closing. Each fails early with a readable reason where integrity
//! (or the apply-time rules, ADR 0033) would not accept the result.

use hdk::prelude::*;
use project_integrity::{EntryTypes, LinkTypes, Outcome, Proposal, ProposalKind, Vote, VoteChoice};
use sp_governance::phases::{next_phase, signal_gate_met, Phase, Shape};
use sp_governance::plans::{required_plan_kind, Leader};
use sp_governance::{tally, Choice, Tally, Threshold, Verdict};

use crate::apply::{self, Standing, State};
use crate::reads::{self, Found};
use crate::{calls, plan_write, plans};

pub fn guest(message: impl Into<String>) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(message.into()))
}

pub fn propose(kind: ProposalKind) -> ExternResult<ActionHash> {
    let membership =
        calls::my_membership()?.ok_or_else(|| guest("Join the project to propose changes."))?;
    let charter =
        calls::base_charter()?.ok_or_else(|| guest("This project has no charter yet."))?;
    if charter.mode == "personal-service" {
        return Err(guest("Personal services don't take proposals."));
    }
    let kind = match kind {
        ProposalKind::Phase { to, .. } => {
            let state = apply::state(&mut Standing::default())?;
            let plan = check_phase_move(&state, &to)?;
            ProposalKind::Phase { to, plan }
        }
        ProposalKind::Plan(plan) => {
            let state = apply::state(&mut Standing::default())?;
            ProposalKind::Plan(plan_write::prepare(&state, plan)?)
        }
        other => other,
    };
    let is_plan = matches!(kind, ProposalKind::Plan(_));
    let members = calls::member_snapshot()?;
    let hash = create_entry(EntryTypes::Proposal(Proposal {
        kind,
        membership,
        members,
    }))?;
    create_link(reads::anchor()?, hash.clone(), LinkTypes::Proposals, ())?;
    // The proposer's own Yes comes with the proposal (web's proposer vote). Plans are
    // assessed first, and their authors vote like everyone else (web's plan flow).
    if !is_plan {
        vote(hash.clone(), VoteChoice::Yes)?;
    }
    Ok(hash)
}

/// A phase move must be the next phase; leaving Proposal needs ≥ 66% demand (ADR 0033), and
/// leaving a plan phase needs an approved leading plan, which the move cites (ADR 0038).
fn check_phase_move(state: &State, to: &str) -> ExternResult<Option<ActionHash>> {
    let shape = Shape {
        mode: state.mode(),
        subtype: state.subtype(),
        include_access: false,
    };
    let current = state.phase();
    if next_phase(shape, current).map(Phase::id) != Some(to) {
        return Err(guest("A project can only move to its next phase."));
    }
    if current == Phase::P1 {
        let s = calls::signal_counts()?;
        if !signal_gate_met(s.demand, s.opposition) {
            return Err(guest(
                "Planning opens once support is above 66% of all signals.",
            ));
        }
    }
    let Some(kind) = required_plan_kind(shape, current) else {
        return Ok(None);
    };
    match plans::leader(&state.approved, kind) {
        Leader::One(plan) => Ok(Some(plan)),
        _ => Err(guest(
            "An approved plan is required before advancing this project phase.",
        )),
    }
}

pub fn vote(proposal: ActionHash, choice: VoteChoice) -> ExternResult<ActionHash> {
    let membership = calls::my_membership()?.ok_or_else(|| guest("Join the project to vote."))?;
    let me = agent_info()?.agent_initial_pubkey;
    if reads::votes(&proposal)?.iter().any(|v| v.author == me) || voted_locally(&proposal)? {
        return Err(guest("You've already voted on this."));
    }
    if reads::outcome(&proposal)?.is_some() {
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

pub fn tally_of(proposal: &Proposal, votes: &[Found<Vote>]) -> Tally {
    let choices: Vec<Choice> = votes
        .iter()
        .filter_map(|v| match v.entry.choice {
            VoteChoice::Yes => Some(Choice::Yes),
            VoteChoice::No => Some(Choice::No),
            VoteChoice::Neutral => None,
        })
        .collect();
    tally(
        &choices,
        u32::try_from(proposal.members.len()).unwrap_or(u32::MAX),
        Threshold::DEFAULT,
    )
}

/// Write the Outcome once the result is settled; returns it when one exists. `fresh` is a vote
/// just written here, which the network may not show yet.
pub fn close(
    proposal_hash: &ActionHash,
    fresh: Option<Found<Vote>>,
) -> ExternResult<Option<ActionHash>> {
    if let Some(existing) = reads::outcome(proposal_hash)? {
        return Ok(Some(existing.hash));
    }
    let Some(proposal) = reads::proposal(proposal_hash)? else {
        return Ok(None);
    };
    let mut votes = reads::votes(proposal_hash)?;
    if let Some(v) = fresh.filter(|f| votes.iter().all(|v| v.author != f.author)) {
        votes.push(v);
    }
    let passed = match tally_of(&proposal.entry, &votes).verdict() {
        Verdict::Open => return Ok(None),
        Verdict::Passed => true,
        Verdict::Rejected => false,
    };
    let outcome = Outcome {
        proposal: proposal_hash.clone(),
        votes: votes.iter().map(|v| v.hash.clone()).collect(),
        passed,
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
