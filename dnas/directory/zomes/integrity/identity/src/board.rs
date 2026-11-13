//! Moderators, the weekly active count and deactivation (ADRs 0095 and 0088): links in the
//! directory, written as yourself (the target is always the author) and never deleted.
//! Candidacies hang from the "board" path; votes from the account voted on; `Active` from
//! its week's path; `Deactivated` from the account itself. Who is a moderator, how many people were
//! active and who is deactivated is replayed by every reader (`sp_board`, `sp_active`); these
//! rules only keep the history honest.

use hdi::prelude::*;

use crate::{invalid, prev_action_of, LinkTypes};

/// Candidacy tags: `[WITHDRAW]` or `[VOLUNTEER]`.
pub const WITHDRAW: u8 = 0;
pub const VOLUNTEER: u8 = 1;
/// Board vote tags: `[BOARD_NO]`, `[BOARD_YES]` or `[BOARD_NEUTRAL]` (the vote withdrawn).
pub const BOARD_NO: u8 = 0;
pub const BOARD_YES: u8 = 1;
pub const BOARD_NEUTRAL: u8 = 2;

/// The path every candidacy hangs from, so readers list the board with one read.
pub const BOARD_PATH: &str = "board";

/// The path an `Active` link for `week` starts from (`sp_active::week_of`).
#[must_use]
pub fn active_path(week: i64) -> Path {
    Path::from(format!("active.{week}"))
}

pub fn validate_create_link(
    link_type: LinkTypes,
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    let me = Some(action.author());
    let base = action.base_address.clone().into_agent_pub_key();
    let target = action.target_address.clone().into_agent_pub_key();
    if target.as_ref() != me {
        return invalid("moderator, active and deactivation links are written as yourself");
    }
    let tag = action.tag.0.as_slice();
    match link_type {
        LinkTypes::BoardCandidacy if action.base_address != board_path()? => {
            invalid("candidacies hang from the board path")
        }
        LinkTypes::BoardCandidacy if matches!(tag, [WITHDRAW | VOLUNTEER]) => valid(),
        LinkTypes::BoardCandidacy => invalid("a candidacy volunteers or withdraws"),
        LinkTypes::BoardVotes if base.is_none() || base.as_ref() == me => {
            invalid("board votes are on someone else's account")
        }
        LinkTypes::BoardVotes if matches!(tag, [BOARD_NO | BOARD_YES | BOARD_NEUTRAL]) => valid(),
        LinkTypes::BoardVotes => invalid("a board vote is yes, no or neutral"),
        LinkTypes::Deactivated if base.as_ref() == me && tag.is_empty() => valid(),
        LinkTypes::Deactivated => invalid("only the account itself deactivates it"),
        _ => validate_active(action),
    }
}

/// One `Active` link per person per week, from that week's path, with no tag.
fn validate_active(action: &TypedAction<CreateLinkData>) -> ExternResult<ValidateCallbackResult> {
    let at = action.timestamp().as_micros();
    let week = sp_active::week_of(at);
    let path = AnyLinkableHash::from(active_path(week).path_entry_hash()?);
    if action.base_address != path || !action.tag.0.is_empty() {
        return invalid("an active link is from its own week's path, with no tag");
    }
    let since = Timestamp::from_micros(sp_active::week_start(at));
    let filter = ChainFilter::until_timestamp(prev_action_of(action.prev_action())?, since);
    let active: ScopedLinkType = LinkTypes::Active.try_into()?;
    let earlier = must_get_agent_activity(action.author().clone(), filter)?
        .into_iter()
        .any(|a| match &a.action.hashed.content.data {
            ActionData::CreateLink(l) => {
                l.zome_index == active.zome_index
                    && l.link_type == active.zome_type
                    && l.base_address == path
            }
            _ => false,
        });
    if earlier {
        return invalid("one active link per person per week");
    }
    valid()
}

/// A chain closes only right after its own `Deactivated` link (ADR 0088): nothing can be
/// written after it, which system validation enforces for every DNA action.
pub fn validate_close_chain(prev: &ActionHash) -> ExternResult<ValidateCallbackResult> {
    let action = must_get_action(prev.clone())?;
    let deactivated: ScopedLinkType = LinkTypes::Deactivated.try_into()?;
    match &action.hashed.content.data {
        ActionData::CreateLink(l)
            if l.zome_index == deactivated.zome_index && l.link_type == deactivated.zome_type =>
        {
            valid()
        }
        _ => invalid("a chain closes only right after deactivating the account"),
    }
}

fn board_path() -> ExternResult<AnyLinkableHash> {
    Ok(Path::from(BOARD_PATH).path_entry_hash()?.into())
}

fn valid() -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
