//! Proposal → Vote → Outcome inside one project (ADR 0009, ADR 0033). Integrity checks what
//! every validator can reproduce. Standing, the signal gate and the full recount happen when a
//! node applies an outcome.

use std::collections::BTreeSet;

use hdi::prelude::*;
use sp_core::limits::LONG_TEXT_MAX;
use sp_core::project::{DESCRIPTION_MAX, LOCATION_LABEL_MAX, TITLE_MAX};
use sp_governance::phases::Phase;
use sp_governance::{tally, Choice, Threshold, Verdict};

use crate::links::is_membership;
use crate::{invalid, EntryTypes, Outcome, Proposal, ProposalKind, Vote, VoteChoice};

/// Large enough for any project; bounds validation work.
pub const SNAPSHOT_MAX: usize = 10_000;

pub fn validate_proposal(
    p: &Proposal,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_kind(&p.kind) {
        return invalid(&reason);
    }
    if let ProposalKind::Phase {
        plan: Some(plan), ..
    } = &p.kind
    {
        if crate::plan::plan_of(plan)?.is_none() {
            return invalid("a phase move cites a plan proposal");
        }
    }
    if let ProposalKind::Link(link) = &p.kind {
        if let Err(reason) = crate::link::check(link)? {
            return invalid(&reason);
        }
    }
    let at = Some(action.timestamp());
    let mirrored = matches!(&p.kind, ProposalKind::Link(l) if l.source.is_some());
    match &p.membership {
        Some(m) if !is_membership(m, action.author(), at)? => {
            return invalid("only members can propose");
        }
        None if !mirrored => return invalid("only members can propose"),
        _ => {}
    }
    let includes_proposer = p.membership.as_ref().is_none_or(|m| p.members.contains(m));
    if p.members.len() > SNAPSHOT_MAX || !includes_proposer {
        return invalid("the member snapshot must include the proposer");
    }
    let mut seen = BTreeSet::new();
    for hash in &p.members {
        let signed = must_get_action(hash.clone())?;
        let member = signed.action().author().clone();
        if !seen.insert(member.clone()) || !is_membership(hash, &member, at)? {
            return invalid(
                "the member snapshot lists one membership per member, made before the proposal",
            );
        }
    }
    Ok(ValidateCallbackResult::Valid)
}

fn check_kind(kind: &ProposalKind) -> Result<(), String> {
    let within =
        |text: &Option<String>, max: usize| text.as_ref().is_none_or(|t| t.chars().count() <= max);
    match kind {
        ProposalKind::Edit {
            title,
            description,
            location,
        } => {
            if title.is_none() && description.is_none() && location.is_none() {
                return Err("an edit must change something".into());
            }
            let title_ok =
                title.as_ref().is_none_or(|t| !t.trim().is_empty()) && within(title, TITLE_MAX);
            let description_ok = description.as_ref().is_none_or(|d| !d.trim().is_empty())
                && within(description, DESCRIPTION_MAX);
            if !(title_ok && description_ok && within(location, LOCATION_LABEL_MAX)) {
                return Err("an edit's fields must fit the charter's limits".into());
            }
            Ok(())
        }
        ProposalKind::Update { body }
            if !body.trim().is_empty() && body.chars().count() <= LONG_TEXT_MAX =>
        {
            Ok(())
        }
        ProposalKind::Update { .. } => Err("an update needs text".into()),
        ProposalKind::Phase { to, reason, .. } => {
            if Phase::parse(to).is_none() {
                return Err("unknown phase".into());
            }
            if reason.trim().is_empty() || reason.chars().count() > LONG_TEXT_MAX {
                return Err("a phase change needs a reason".into());
            }
            Ok(())
        }
        ProposalKind::Plan(plan) => crate::plan::check(plan),
        ProposalKind::Return { to, reason } => {
            if Phase::parse(to).is_none() {
                return Err("unknown phase".into());
            }
            if reason.trim().is_empty() || reason.chars().count() > LONG_TEXT_MAX {
                return Err("a return needs a reason".into());
            }
            Ok(())
        }
        // Needs the DNA's seed and the cited create half: `link::check`.
        ProposalKind::Link(_) => Ok(()),
    }
}

pub fn validate_vote(
    v: &Vote,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    let Some(EntryTypes::Proposal(_)) =
        crate::typed_entry(&must_get_valid_record(v.proposal.clone())?)?
    else {
        return invalid("a vote must be on a proposal");
    };
    if !is_membership(&v.membership, action.author(), Some(action.timestamp()))? {
        return invalid("only members can vote");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// The claimed result must be what the cited votes (one per voter, neutrals not counted) give
/// against the proposal's snapshot, enlarged by the members its cited challenges add (ADR 0040).
pub fn validate_outcome(
    o: &Outcome,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    let proposal_record = must_get_valid_record(o.proposal.clone())?;
    let Some(EntryTypes::Proposal(proposal)) = crate::typed_entry(&proposal_record)? else {
        return invalid("an outcome must close a proposal");
    };
    let mut voters = BTreeSet::new();
    let mut choices = Vec::new();
    for hash in &o.votes {
        let record = must_get_valid_record(hash.clone())?;
        let Some(EntryTypes::Vote(vote)) = crate::typed_entry(&record)? else {
            return invalid("an outcome cites only votes");
        };
        let late = record.action().timestamp() > action.timestamp();
        if vote.proposal != o.proposal || late || !voters.insert(record.action().author().clone()) {
            return invalid("an outcome cites one earlier vote per voter on its proposal");
        }
        match vote.choice {
            VoteChoice::Yes => choices.push(Choice::Yes),
            VoteChoice::No => choices.push(Choice::No),
            VoteChoice::Neutral => {}
        }
    }
    let added = match crate::challenge::added(&o.proposal, &o.challenges, action.timestamp())? {
        Ok(added) => added,
        Err(reason) => return invalid(reason),
    };
    let population = u32::try_from(proposal.members.len() + added).unwrap_or(u32::MAX);
    let verdict = tally(&choices, population, Threshold::DEFAULT).verdict();
    let claimed = if o.passed {
        Verdict::Passed
    } else {
        Verdict::Rejected
    };
    if verdict == claimed {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid("the outcome's result doesn't match its votes")
    }
}
