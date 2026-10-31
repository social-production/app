//! Link rules: membership and signals are links from a space to their author (ADR 0030, 0065);
//! governance links point at records validated on their own.

use hdi::prelude::*;

use crate::space::is_space;
use crate::{invalid, LinkTypes, DEMAND, OPPOSE};

pub fn validate_create(
    link_type: LinkTypes,
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    let target_is_author =
        action.target_address.clone().into_agent_pub_key().as_ref() == Some(action.author());
    match link_type {
        LinkTypes::ProjectCharter => crate::charter::validate_anchor(action),
        LinkTypes::Members | LinkTypes::Signals if !is_space(&action.base_address)? => {
            invalid("people join or signal a space")
        }
        LinkTypes::Members if target_is_author => Ok(ValidateCallbackResult::Valid),
        LinkTypes::Signals if crate::organized()? => {
            invalid("organizer-controlled events don't use signals")
        }
        LinkTypes::Signals if target_is_author => match action.tag.0.as_slice() {
            [DEMAND | OPPOSE] => Ok(ValidateCallbackResult::Valid),
            _ => invalid("a signal is demand or opposition"),
        },
        LinkTypes::Members | LinkTypes::Signals => {
            invalid("agents can only join or signal as themselves")
        }
        LinkTypes::Assignments => crate::activity::validate_assignment(action),
        LinkTypes::Proposals
        | LinkTypes::ProposalToVotes
        | LinkTypes::ProposalToOutcomes
        | LinkTypes::ProposalToRatings
        | LinkTypes::ProposalToChallenges
        | LinkTypes::Values
        | LinkTypes::ValueToImportance
        | LinkTypes::Activities
        | LinkTypes::ActivityToSuggestions
        | LinkTypes::ActivityToRatings
        | LinkTypes::Organizers => Ok(ValidateCallbackResult::Valid),
        LinkTypes::Recent if is_space(&action.target_address)? => Ok(ValidateCallbackResult::Valid),
        LinkTypes::Recent => invalid("a recent-activity link points at a space"),
        LinkTypes::ScopeItems => crate::scope_item::validate(action),
    }
}

/// `hash` is a `Members` link the agent made for itself before `before` (or at any time when
/// `None`), in `space` when one is given: a membership counts only in its own space (ADR 0065).
pub fn is_membership(
    hash: &ActionHash,
    agent: &AgentPubKey,
    before: Option<Timestamp>,
    space: Option<&ActionHash>,
) -> ExternResult<bool> {
    let signed = must_get_action(hash.clone())?;
    let action = signed.action();
    let members: ScopedLinkType = LinkTypes::Members.try_into()?;
    let ActionData::CreateLink(link) = &action.data else {
        return Ok(false);
    };
    let is_members = link.zome_index == members.zome_index && link.link_type == members.zome_type;
    let in_space = space.is_none_or(|s| link.base_address == AnyLinkableHash::from(s.clone()));
    Ok(is_members
        && in_space
        && action.author() == agent
        && before.is_none_or(|t| action.timestamp() < t))
}

/// The space a `Members` link belongs to (its base), or `None` when `hash` isn't one.
pub fn membership_space(hash: &ActionHash) -> ExternResult<Option<ActionHash>> {
    let signed = must_get_action(hash.clone())?;
    let members: ScopedLinkType = LinkTypes::Members.try_into()?;
    Ok(match &signed.action().data {
        ActionData::CreateLink(link)
            if link.zome_index == members.zome_index && link.link_type == members.zome_type =>
        {
            link.base_address.clone().into_action_hash()
        }
        _ => None,
    })
}
