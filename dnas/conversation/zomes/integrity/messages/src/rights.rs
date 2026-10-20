//! Who may do what in a chat, from the records a write cites (ADR 0043, 0045). Validators
//! need no lookups: a cited record was itself validated, so one step of the chain suffices.

use hdi::prelude::*;

use crate::{is_type, Invite, Leave, UnitEntryTypes};

fn decode<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    record: &Record,
) -> ExternResult<Option<T>> {
    record
        .entry()
        .to_app_option::<T>()
        .map_err(|e| wasm_error!(e))
}

/// `author` manages the chat: they are its creator (`by` is `None`), or `by` is the valid
/// `Leave` by which the previous manager handed the group to them.
pub fn manages(
    author: &AgentPubKey,
    by: Option<&ActionHash>,
    creator: &AgentPubKey,
) -> ExternResult<bool> {
    let Some(by) = by else {
        return Ok(author == creator);
    };
    let record = must_get_valid_record(by.clone())?;
    if !is_type(record.action(), UnitEntryTypes::Leave)? {
        return Ok(false);
    }
    Ok(decode::<Leave>(&record)?.is_some_and(|l| l.successor.as_ref() == Some(author)))
}

/// `author` was brought into the chat: they are its creator (`invite` is `None`), or
/// `invite` is a valid invitation naming them. A valid invitation was written by a manager.
pub fn member(
    author: &AgentPubKey,
    invite: Option<&ActionHash>,
    creator: &AgentPubKey,
) -> ExternResult<bool> {
    let Some(invite) = invite else {
        return Ok(author == creator);
    };
    let record = must_get_valid_record(invite.clone())?;
    if !is_type(record.action(), UnitEntryTypes::Invite)? {
        return Ok(false);
    }
    Ok(decode::<Invite>(&record)?.is_some_and(|i| &i.agent == author))
}

/// The author of `message`, when it is a message.
pub fn message_author(message: &ActionHash) -> ExternResult<Option<AgentPubKey>> {
    let record = must_get_valid_record(message.clone())?;
    Ok(
        is_type(record.action(), UnitEntryTypes::Message)?
            .then(|| record.action().author().clone()),
    )
}
