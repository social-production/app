//! Challenges to a proposal's count (docs/GOVERNANCE.md, ADR 0040). Integrity checks what every
//! validator can reproduce: each cited membership is a `Members` link made before the proposal,
//! by someone the snapshot doesn't count, named once. Whether that member had left before the
//! proposal, and whether the challenge came within the window, are checked when applying.

use std::collections::BTreeSet;

use hdi::prelude::*;

use crate::links::is_membership;
use crate::{invalid, Challenge, EntryTypes, SNAPSHOT_MAX};

pub fn validate(
    c: &Challenge,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    let record = must_get_valid_record(c.proposal.clone())?;
    let Some(EntryTypes::Proposal(proposal)) = crate::typed_entry(&record)? else {
        return invalid("a challenge must be on a proposal");
    };
    let proposed_at = record.action().timestamp();
    if action.timestamp() < proposed_at {
        return invalid("a challenge comes after its proposal");
    }
    if !is_membership(&c.membership, action.author(), Some(action.timestamp()))? {
        return invalid("only members can challenge a count");
    }
    if c.omitted.is_empty() || c.omitted.len() > SNAPSHOT_MAX {
        return invalid("a challenge names at least one omitted member");
    }
    let counted = authors(&proposal.members)?;
    let mut named = BTreeSet::new();
    for hash in &c.omitted {
        let member = must_get_action(hash.clone())?.action().author().clone();
        let fresh = !counted.contains(&member) && named.insert(member.clone());
        if !fresh || !is_membership(hash, &member, Some(proposed_at))? {
            return invalid(
                "a challenge names memberships from before the proposal that its count left out, once each",
            );
        }
    }
    Ok(ValidateCallbackResult::Valid)
}

fn authors(hashes: &[ActionHash]) -> ExternResult<BTreeSet<AgentPubKey>> {
    hashes
        .iter()
        .map(|h| Ok(must_get_action(h.clone())?.action().author().clone()))
        .collect()
}

/// How many members an Outcome's cited challenges add to its proposal's snapshot, or why they
/// can't be cited: each must be a valid challenge on the same proposal, made before the Outcome.
pub fn added(
    proposal: &ActionHash,
    challenges: &[ActionHash],
    outcome_at: Timestamp,
) -> ExternResult<Result<usize, &'static str>> {
    let mut seen = BTreeSet::new();
    let mut members = BTreeSet::new();
    for hash in challenges {
        let record = must_get_valid_record(hash.clone())?;
        let Some(EntryTypes::Challenge(c)) = crate::typed_entry(&record)? else {
            return Ok(Err("an outcome cites only challenges as challenges"));
        };
        let late = record.action().timestamp() > outcome_at;
        if &c.proposal != proposal || late || !seen.insert(hash.clone()) {
            return Ok(Err(
                "an outcome cites each earlier challenge on its proposal once",
            ));
        }
        for omitted in &c.omitted {
            members.insert(must_get_action(omitted.clone())?.action().author().clone());
        }
    }
    Ok(Ok(members.len()))
}
