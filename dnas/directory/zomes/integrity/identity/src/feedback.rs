//! Platform feedback (ADR 0054): bug reports and suggestions, with up/down votes as links
//! from the feedback to the voter (the newest per voter counts, as project signals do).

use hdi::prelude::*;
use sp_core::limits::{text_ok, SHORT_TEXT_MAX};

use crate::{invalid, typed_entry, EntryTypes, Feedback};

/// web's limit on a feedback description.
pub const FEEDBACK_DESCRIPTION_MAX: usize = 5_000;
pub const VOTE_UP: u8 = 0;
pub const VOTE_DOWN: u8 = 1;

pub fn validate_fields(f: &Feedback) -> ExternResult<ValidateCallbackResult> {
    if !text_ok(&f.title, SHORT_TEXT_MAX) {
        return invalid("feedback needs a title of at most 200 characters");
    }
    if !text_ok(&f.description, FEEDBACK_DESCRIPTION_MAX) {
        return invalid("feedback needs a description of at most 5000 characters");
    }
    Ok(ValidateCallbackResult::Valid)
}

fn is_feedback(hash: Option<ActionHash>) -> ExternResult<Option<AgentPubKey>> {
    let Some(hash) = hash else { return Ok(None) };
    let record = must_get_valid_record(hash)?;
    Ok(
        matches!(typed_entry(&record)?, Some(EntryTypes::Feedback(_)))
            .then(|| record.action().author().clone()),
    )
}

/// The "feedback" anchor links each item from its author; a vote names its voter.
pub fn validate_listing_link(
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    match is_feedback(action.target_address.clone().into_action_hash())? {
        Some(author) if &author == action.author() => Ok(ValidateCallbackResult::Valid),
        _ => invalid("only a feedback item's author lists it"),
    }
}

pub fn validate_vote_link(
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    let voter = action.target_address.clone().into_agent_pub_key();
    if voter.as_ref() != Some(action.author()) {
        return invalid("agents can only vote as themselves");
    }
    if !matches!(action.tag.0.as_slice(), [VOTE_UP | VOTE_DOWN]) {
        return invalid("a feedback vote is up or down");
    }
    if is_feedback(action.base_address.clone().into_action_hash())?.is_none() {
        return invalid("votes are on feedback items");
    }
    Ok(ValidateCallbackResult::Valid)
}
