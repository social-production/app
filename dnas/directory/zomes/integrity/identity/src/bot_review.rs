//! Bot marks, bot review votes and re-review requests (ADR 0074): links on the reviewed
//! account. Who counts and what the review decided is replayed by every reader
//! (`sp_bot_review`); these rules only keep the history honest.

use hdi::prelude::*;

use crate::{invalid, LinkTypes};

/// Mark tags: `[MARK_CLEAR]` or `[MARK_BOT]`; vote tags: `[VOTE_REAL]` or `[VOTE_BOT]`.
pub const MARK_CLEAR: u8 = 0;
pub const MARK_BOT: u8 = 1;
pub const VOTE_REAL: u8 = 0;
pub const VOTE_BOT: u8 = 1;

/// Account → marker or voter (as themselves, never on themselves); a re-review request is the
/// account linking to itself.
pub fn validate_create_link(
    link_type: LinkTypes,
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    let account = action.base_address.clone().into_agent_pub_key();
    let who = action.target_address.clone().into_agent_pub_key();
    if who.as_ref() != Some(action.author()) {
        return invalid("bot marks and review votes are written as yourself");
    }
    let own = account.as_ref() == Some(action.author());
    match link_type {
        LinkTypes::ReReviews if own && action.tag.0.is_empty() => Ok(ValidateCallbackResult::Valid),
        LinkTypes::ReReviews => invalid("only the account itself asks for a re-review"),
        _ if account.is_none() || own => invalid("bot marks and votes are on someone else"),
        _ if matches!(action.tag.0.as_slice(), [MARK_CLEAR | MARK_BOT]) => {
            Ok(ValidateCallbackResult::Valid)
        }
        _ => invalid("a bot mark is bot or clear; a review vote is bot or real"),
    }
}
