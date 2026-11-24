//! Challenging a count that left members out (docs/GOVERNANCE.md, ADR 0040).

use hdk::prelude::*;
use project_integrity::{Challenge, EntryTypes, LinkTypes};

use crate::calls;
use crate::count::count;
use crate::decide::Standing;
use crate::reads;
use crate::roll::Roll;
use crate::write::guest;

/// Cite every member the proposal's count leaves out (docs/GOVERNANCE.md, ADR 0040). The
/// proposal reopens with the larger population; it closes again once its votes decide it.
pub fn challenge(proposal: ActionHash) -> ExternResult<ActionHash> {
    let membership = calls::my_membership()?
        .ok_or_else(|| guest("Join the project to challenge a decision."))?;
    let found = reads::proposal(&proposal)?
        .ok_or_else(|| guest("This proposal hasn't reached your node yet."))?;
    let roll = Roll::load()?;
    let count = count(&found, &roll)?;
    if !count.open_to_challenge(sys_time()?) {
        return Err(guest("The time to challenge this decision has passed."));
    }
    let omitted = count.full_missing(&found, &roll, &mut Standing::default())?;
    if omitted.is_empty() {
        return Err(guest(
            "Every member from when this was proposed is already counted.",
        ));
    }
    let hash = create_entry(EntryTypes::Challenge(Challenge {
        proposal: proposal.clone(),
        membership,
        omitted,
    }))?;
    create_link(proposal, hash.clone(), LinkTypes::ProposalToChallenges, ())?;
    Ok(hash)
}
