//! Entry and link checks for project talk (ADR 0038 chat, ADR 0041 threads).

use hdi::prelude::*;
use sp_core::chat::check_message;
use sp_core::discussion::{check_comment, check_thread, vote_direction};

use crate::{invalid, is_comment, is_message, is_thread, ChatMessage, Comment, LinkTypes, Thread};

pub fn validate_message(m: &ChatMessage) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_message(&m.body) {
        return invalid(&reason);
    }
    if let Some(parent) = &m.reply_to {
        let record = must_get_valid_record(parent.clone())?;
        if !is_message(record.action())? {
            return invalid("a reply answers a chat message");
        }
    }
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_thread(t: &Thread) -> ExternResult<ValidateCallbackResult> {
    match check_thread(&t.title, &t.body) {
        Ok(()) => Ok(ValidateCallbackResult::Valid),
        Err(reason) => invalid(&reason),
    }
}

/// A comment belongs to a thread, and a reply to a comment of the same thread (as on web).
pub fn validate_comment(c: &Comment) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_comment(&c.body) {
        return invalid(&reason);
    }
    let thread = must_get_valid_record(c.thread.clone())?;
    if !is_thread(thread.action())? {
        return invalid("a comment belongs to a thread");
    }
    if let Some(parent) = &c.parent {
        let record = must_get_valid_record(parent.clone())?;
        let same_thread = decode_comment(&record)?.is_some_and(|p| p.thread == c.thread);
        if !same_thread {
            return invalid("a reply answers a comment in the same thread");
        }
    }
    Ok(ValidateCallbackResult::Valid)
}

/// The comment a record holds, if it creates one.
pub fn decode_comment(record: &Record) -> ExternResult<Option<Comment>> {
    if !is_comment(record.action())? {
        return Ok(None);
    }
    record
        .entry()
        .to_app_option::<Comment>()
        .map_err(|e| wasm_error!(e))
}

/// People list only their own talk, so deleting their link takes it out for good; votes are
/// cast only as yourself.
pub fn validate_link(
    link_type: LinkTypes,
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    match link_type {
        LinkTypes::ChatMessages => own_target(action, is_message, "chat message"),
        LinkTypes::Threads => own_target(action, is_thread, "thread"),
        LinkTypes::ThreadComments => validate_thread_comment(action),
        LinkTypes::Votes => validate_vote(action),
    }
}

/// The link points at a `kind` record its own author made.
fn own_target(
    action: &TypedAction<CreateLinkData>,
    kind: fn(&Action) -> ExternResult<bool>,
    label: &str,
) -> ExternResult<ValidateCallbackResult> {
    let Some(target) = action.target_address.clone().into_action_hash() else {
        return invalid(&format!("this link points at a {label}"));
    };
    let signed = must_get_action(target)?;
    if kind(signed.action())? && signed.action().author() == action.author() {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid(&format!("people list only their own {label}s"))
    }
}

fn validate_thread_comment(
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    let Some(target) = action.target_address.clone().into_action_hash() else {
        return invalid("a thread's link points at a comment");
    };
    let record = must_get_valid_record(target)?;
    let Some(comment) = decode_comment(&record)? else {
        return invalid("a thread's link points at a comment");
    };
    if record.action().author() != action.author() {
        return invalid("people list only their own comments");
    }
    if action.base_address != AnyLinkableHash::from(comment.thread) {
        return invalid("a comment is listed under its own thread");
    }
    Ok(ValidateCallbackResult::Valid)
}

fn validate_vote(action: &TypedAction<CreateLinkData>) -> ExternResult<ValidateCallbackResult> {
    if action.target_address.clone().into_agent_pub_key().as_ref() != Some(action.author()) {
        return invalid("people vote only as themselves");
    }
    if vote_direction(&action.tag.0) == 0 {
        return invalid("a vote is up or down");
    }
    let Some(base) = action.base_address.clone().into_action_hash() else {
        return invalid("votes go on threads and comments");
    };
    let voted = must_get_action(base)?;
    if is_thread(voted.action())? || is_comment(voted.action())? {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid("votes go on threads and comments")
    }
}
