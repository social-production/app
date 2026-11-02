//! Reports (ADR 0071): web's ⋮ Report on a project, event, help request, thread or post. A
//! report names the item (its space's charter hash), a reason and what's wrong; votes are
//! links from the report to the voter. What a report decides is replayed by every reader
//! from these records (`sp_moderation::replay`), so nothing here counts votes.

use hdi::prelude::*;
use sp_moderation::{check_report, is_vote_tag};

use crate::{invalid, is_type, UnitEntryTypes};

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Report {
    /// What is reported: a space's charter create hash (a chat message, later).
    pub target: ActionHash,
    /// web's reason id: `spam` or `serious-harm`.
    pub reason: String,
    pub description: String,
}

/// A known reason and some text, about something that exists and isn't the reporter's own
/// (web's "You can't report yourself").
pub fn validate_report(r: &Report, author: &AgentPubKey) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_report(&r.reason, &r.description) {
        return invalid(&reason);
    }
    let target = must_get_action(r.target.clone())?;
    if target.action().author() == author {
        return invalid("you can't report yourself");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// The reporter lists their report on the item it reports.
pub fn validate_listing(
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    let Some((record, report)) = report_at(&action.target_address)? else {
        return invalid("a report link points at a report");
    };
    if record.action().author() != action.author() {
        return invalid("people list only their own reports");
    }
    if AnyLinkableHash::from(report.target) != action.base_address {
        return invalid("a report is listed on what it reports");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// People vote yes or no as themselves, never on a report about their own item.
pub fn validate_vote(action: &TypedAction<CreateLinkData>) -> ExternResult<ValidateCallbackResult> {
    if action.target_address.clone().into_agent_pub_key().as_ref() != Some(action.author()) {
        return invalid("people vote as themselves");
    }
    if !is_vote_tag(&action.tag.0) {
        return invalid("a report vote is yes or no");
    }
    let Some((_, report)) = report_at(&action.base_address)? else {
        return invalid("a report vote is on a report");
    };
    if must_get_action(report.target)?.action().author() == action.author() {
        return invalid("you can't vote on a report about your own post");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// The report at `address`, if it is one.
fn report_at(address: &AnyLinkableHash) -> ExternResult<Option<(Record, Report)>> {
    let Some(hash) = address.clone().into_action_hash() else {
        return Ok(None);
    };
    let record = must_get_valid_record(hash)?;
    if !is_type(record.action(), UnitEntryTypes::Report)? {
        return Ok(None);
    }
    let report = record
        .entry()
        .to_app_option::<Report>()
        .map_err(|e| wasm_error!(e))?;
    Ok(report.map(|r| (record, r)))
}
