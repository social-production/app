//! Link rules: membership and signals are links whose target is their author (ADR 0030);
//! governance links point at records validated on their own.

use hdi::prelude::*;

use crate::charter::by_creator;
use crate::{invalid, LinkTypes, DEMAND, OPPOSE};

pub fn validate_create(
    link_type: LinkTypes,
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    let target_is_author =
        action.target_address.clone().into_agent_pub_key().as_ref() == Some(action.author());
    match link_type {
        LinkTypes::ProjectCharter => by_creator(action.author(), || Ok(())),
        LinkTypes::Members if target_is_author => Ok(ValidateCallbackResult::Valid),
        LinkTypes::Signals if target_is_author => match action.tag.0.as_slice() {
            [DEMAND | OPPOSE] => Ok(ValidateCallbackResult::Valid),
            _ => invalid("a signal is demand or opposition"),
        },
        LinkTypes::Members | LinkTypes::Signals => {
            invalid("agents can only join or signal as themselves")
        }
        LinkTypes::Proposals
        | LinkTypes::ProposalToVotes
        | LinkTypes::ProposalToOutcomes
        | LinkTypes::ProposalToRatings => Ok(ValidateCallbackResult::Valid),
    }
}

/// `hash` is a `Members` link the agent made for itself before `before` (or at any time when `None`).
pub fn is_membership(
    hash: &ActionHash,
    agent: &AgentPubKey,
    before: Option<Timestamp>,
) -> ExternResult<bool> {
    let signed = must_get_action(hash.clone())?;
    let action = signed.action();
    let members: ScopedLinkType = LinkTypes::Members.try_into()?;
    let ActionData::CreateLink(link) = &action.data else {
        return Ok(false);
    };
    let is_members = link.zome_index == members.zome_index && link.link_type == members.zome_type;
    Ok(is_members && action.author() == agent && before.is_none_or(|t| action.timestamp() < t))
}
