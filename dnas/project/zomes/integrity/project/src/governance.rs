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
    if let Err(reason) = check_space(p, action.author())? {
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
    if let Err(reason) = check_cited(p)? {
        return invalid(&reason);
    }
    let at = Some(action.timestamp());
    let mirrored = matches!(&p.kind, ProposalKind::Link(l) if l.source.is_some());
    if !mirrored {
        let grant = p.organizer.as_ref();
        if let Some(refused) =
            crate::organizer::require_in_organized(action.author(), grant, "propose")?
        {
            return Ok(refused);
        }
    }
    if crate::organized()?
        && matches!(&p.kind, ProposalKind::Return { to, .. } if to.as_str() == Phase::P1.id())
    {
        return invalid("an organizer-controlled event can't return to Proposal");
    }
    match &p.membership {
        Some(m) if !is_membership(m, action.author(), at, Some(&p.space))? => {
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
        if !seen.insert(member.clone()) || !is_membership(hash, &member, at, Some(&p.space))? {
            return invalid(
                "the member snapshot lists one membership per member, made before the proposal",
            );
        }
    }
    Ok(ValidateCallbackResult::Valid)
}

/// What a link or a software request cites must exist as it says (ADR 0042, 0086).
fn check_cited(p: &Proposal) -> ExternResult<Result<(), String>> {
    match &p.kind {
        ProposalKind::Link(link) => crate::link::check(link, &p.space),
        ProposalKind::Software(request) => crate::software::check_cited(request, &p.space),
        _ => Ok(Ok(())),
    }
}

fn check_kind(kind: &ProposalKind) -> Result<(), String> {
    let within =
        |text: &Option<String>, max: usize| text.as_ref().is_none_or(|t| t.chars().count() <= max);
    match kind {
        ProposalKind::Edit {
            title,
            description,
            location,
            place,
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
            crate::place::check_optional(place.as_deref(), location.as_deref())
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
        ProposalKind::Software(request) => crate::software::check(request),
        ProposalKind::Return { to, reason } => {
            if Phase::parse(to).is_none() {
                return Err("unknown phase".into());
            }
            if reason.trim().is_empty() || reason.chars().count() > LONG_TEXT_MAX {
                return Err("a return needs a reason".into());
            }
            Ok(())
        }
        // A link needs the DNA's seed and the cited create half (`link::check`); a removal
        // needs the space's mode (`check_scope_kind`).
        // Request settings need the space's mode too (whether a reason is needed).
        ProposalKind::Link(_)
        | ProposalKind::RemoveMember { .. }
        | ProposalKind::RequestSettings { .. } => Ok(()),
    }
}

/// What the space's mode allows, and who may change a personal service's request settings.
fn check_space(p: &Proposal, author: &AgentPubKey) -> ExternResult<Result<(), String>> {
    let Some((charter, founder)) = crate::charter_at(&p.space)? else {
        return Ok(Err("a proposal decides for a space".into()));
    };
    if sp_scope::is_platform(&charter.mode, &charter.title) {
        return Ok(Err("the Platform channel takes no proposals".into()));
    }
    if let Err(reason) = check_scope_kind(&charter.mode, &p.kind) {
        return Ok(Err(reason));
    }
    if personal_settings(&charter.mode, &p.kind) && author != &founder {
        return Ok(Err(
            "only a personal service's creator changes its request settings".into(),
        ));
    }
    Ok(Ok(()))
}

/// A personal service's creator saves its request settings without a vote (ADR 0076).
fn personal_settings(mode: &str, kind: &ProposalKind) -> bool {
    mode == "personal-service" && matches!(kind, ProposalKind::RequestSettings { .. })
}

pub fn validate_vote(
    v: &Vote,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    let Some(EntryTypes::Proposal(proposal)) =
        crate::typed_entry(&must_get_valid_record(v.proposal.clone())?)?
    else {
        return invalid("a vote must be on a proposal");
    };
    let at = Some(action.timestamp());
    if !is_membership(&v.membership, action.author(), at, Some(&proposal.space))? {
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
    // Organizers decide at once (ADR 0061): their proposals pass without votes.
    let mirrored = matches!(&proposal.kind, ProposalKind::Link(l) if l.source.is_some());
    if o.passed && o.votes.is_empty() && !mirrored && crate::organized()? {
        return Ok(ValidateCallbackResult::Valid);
    }
    // A personal service's creator saves request settings at once (ADR 0076).
    if o.passed
        && o.votes.is_empty()
        && matches!(proposal.kind, ProposalKind::RequestSettings { .. })
    {
        let Some((charter, founder)) = crate::charter_at(&proposal.space)? else {
            return invalid("a proposal decides for a space");
        };
        let by_founder =
            proposal_record.action().author() == &founder && action.author() == &founder;
        if personal_settings(&charter.mode, &proposal.kind) && by_founder {
            return Ok(ValidateCallbackResult::Valid);
        }
    }
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

/// Channels and communities (ADR 0068) take removals; their handle (the title) never changes,
/// since items name them by it. Nothing else takes removals.
pub fn check_scope_kind(mode: &str, kind: &ProposalKind) -> Result<(), String> {
    let scope = sp_scope::is_scope(mode);
    match kind {
        ProposalKind::RemoveMember { .. } if !scope => {
            Err("only channels and communities remove members".into())
        }
        ProposalKind::Edit { title: Some(_), .. } if scope => {
            Err("a channel's or community's name is its handle and stays".into())
        }
        ProposalKind::RequestSettings {
            mode: requests,
            reason,
            ..
        } => sp_service::check_settings(
            crate::applied::typed_as(mode, "collective-service"),
            requests,
            reason,
        ),
        ProposalKind::Software(_)
            if !sp_software::takes_software(crate::applied::typed_as(mode, "productive")) =>
        {
            Err("only productive projects and collective services take software proposals".into())
        }
        _ => Ok(()),
    }
}
