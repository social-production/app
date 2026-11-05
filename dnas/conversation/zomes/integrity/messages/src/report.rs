//! Reports on direct and group chat messages (ADR 0075, web's `message` target): a chat
//! member reports someone else's message; members vote yes (take it down) or no. Every
//! reader replays what the votes decide (`sp_moderation::replay` with web's 1:1 path), so
//! nothing here counts votes. Both entries cite the writer's invitation, so validators
//! check membership without lookups beyond the cited records.
//!
//! Also here: `Declined`, a private note in the inbox that you declined an invitation to a
//! private event or community (ADR 0061), so the invite stays hidden on every device that
//! runs your inbox cell.

use hdi::prelude::*;
use sp_moderation::check_report;

use crate::rights::{decode, member, message_author};
use crate::{invalid, is_type, UnitEntryTypes};

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Report {
    pub message: ActionHash,
    /// web's reason id: `spam` or `serious-harm`.
    pub reason: String,
    pub description: String,
    /// The reporter's invitation; `None` for the chat's creator.
    pub invite: Option<ActionHash>,
}

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ReportVote {
    pub report: ActionHash,
    pub yes: bool,
    /// The voter's invitation; `None` for the chat's creator.
    pub invite: Option<ActionHash>,
}

/// Private, inbox only: the seed of an invitation you declined.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Declined {
    pub seed: String,
}

/// A known reason and some text, by a member, about someone else's message.
pub fn validate_report(
    r: &Report,
    author: &AgentPubKey,
    creator: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_report(&r.reason, &r.description) {
        return invalid(&reason);
    }
    match message_author(&r.message)? {
        None => return invalid("a report is about a message"),
        Some(a) if &a == author => return invalid("you can't report yourself"),
        Some(_) => {}
    }
    if !member(author, r.invite.as_ref(), creator)? {
        return invalid("only people in the chat report its messages");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// A member's yes or no on a report, never on one about their own message.
pub fn validate_vote(
    v: &ReportVote,
    author: &AgentPubKey,
    creator: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    let record = must_get_valid_record(v.report.clone())?;
    if !is_type(record.action(), UnitEntryTypes::Report)? {
        return invalid("a report vote is on a report");
    }
    let Some(report) = decode::<Report>(&record)? else {
        return invalid("a report vote is on a report");
    };
    if message_author(&report.message)?.as_ref() == Some(author) {
        return invalid("you can't vote on a report about your own message");
    }
    if !member(author, v.invite.as_ref(), creator)? {
        return invalid("only people in the chat vote on its reports");
    }
    Ok(ValidateCallbackResult::Valid)
}
