//! Photos and files (ADR 0049): a message's attachments name chunks its author wrote, and each
//! chunk is a bounded part written by someone in the chat.

use hdi::prelude::*;
use sp_attachment::{check_attachment, check_body, check_chunk};

use crate::chat::is_type;
use crate::rights::member;
use crate::{invalid, Chunk, Message, UnitEntryTypes};

/// The body fits beside the attachments, each is described within web's limits, and each
/// part is one of the author's own chunks (whose size its own validation checks).
pub fn validate_message_files(
    m: &Message,
    author: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_body(&m.body, m.attachments.len()) {
        return invalid(&reason);
    }
    for a in &m.attachments {
        let size = usize::try_from(a.size).unwrap_or(usize::MAX);
        if let Err(reason) = check_attachment(&a.kind, &a.name, &a.mime, size, a.chunks.len()) {
            return invalid(&reason);
        }
        for chunk in &a.chunks {
            let signed = must_get_action(chunk.clone())?;
            if !is_type(signed.action(), UnitEntryTypes::Chunk)?
                || signed.action().author() != author
            {
                return invalid("a message's files are parts its author wrote");
            }
        }
    }
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_chunk(
    c: &Chunk,
    author: &AgentPubKey,
    creator: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_chunk(c.bytes.len()) {
        return invalid(&reason);
    }
    if member(author, c.invite.as_ref(), creator)? {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid("only people invited to the chat share files in it")
    }
}
