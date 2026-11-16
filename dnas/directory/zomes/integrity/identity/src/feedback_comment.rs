//! Comments on platform feedback (Plan, Phase and Feedback Screens §7; owner approved the
//! directory reset 2026-11-15): flat comments that may name the comment they answer, linked
//! from the item; likes are links from a comment to the person who likes it.

use hdi::prelude::*;
use sp_core::limits::text_ok;

use crate::{invalid, typed_entry, EntryTypes};

/// A comment's length limit (a post comment's).
pub const FEEDBACK_COMMENT_MAX: usize = 2_000;

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct FeedbackComment {
    pub feedback: ActionHash,
    pub body: String,
    /// The comment this one answers, on the same item.
    pub reply_to: Option<ActionHash>,
}

fn comment_of(hash: Option<ActionHash>) -> ExternResult<Option<(FeedbackComment, AgentPubKey)>> {
    let Some(hash) = hash else { return Ok(None) };
    let record = must_get_valid_record(hash)?;
    Ok(match typed_entry(&record)? {
        Some(EntryTypes::FeedbackComment(c)) => Some((c, record.action().author().clone())),
        _ => None,
    })
}

pub fn validate_create(c: &FeedbackComment) -> ExternResult<ValidateCallbackResult> {
    if !text_ok(&c.body, FEEDBACK_COMMENT_MAX) {
        return invalid("a comment needs text of at most 2000 characters");
    }
    let item = must_get_valid_record(c.feedback.clone())?;
    if !matches!(typed_entry(&item)?, Some(EntryTypes::Feedback(_))) {
        return invalid("comments are on feedback items");
    }
    if let Some(parent) = &c.reply_to {
        match comment_of(Some(parent.clone()))? {
            Some((p, _)) if p.feedback == c.feedback => {}
            _ => return invalid("a reply answers a comment on the same item"),
        }
    }
    Ok(ValidateCallbackResult::Valid)
}

/// Item → comment, written by the comment's author, from the item it is on.
pub fn validate_comment_link(
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    match comment_of(action.target_address.clone().into_action_hash())? {
        Some((c, author))
            if &author == action.author()
                && action.base_address.clone().into_action_hash().as_ref() == Some(&c.feedback) =>
        {
            Ok(ValidateCallbackResult::Valid)
        }
        _ => invalid("a comment is listed on its own item by its author"),
    }
}

/// Comment → the person who likes it, written by that person.
pub fn validate_like_link(
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    if action.target_address.clone().into_agent_pub_key().as_ref() != Some(action.author()) {
        return invalid("agents can only like as themselves");
    }
    if comment_of(action.base_address.clone().into_action_hash())?.is_none() {
        return invalid("likes are on feedback comments");
    }
    Ok(ValidateCallbackResult::Valid)
}
