//! Service requests (ADR 0076, web's personal-service requests): the person asking writes a
//! `ServiceRequest` in their direct chat with the service's creator, so only those two see
//! it; `RequestStatus` lines settle it. The person asking cancels; the other person accepts
//! or declines. Every reader replays the decisions (`sp_service::replay`).

use hdi::prelude::*;
use sp_core::conversation::ChatKind;
use sp_service::{check_request, check_status, may_decide};

use crate::rights::{decode, member};
use crate::{invalid, is_type, UnitEntryTypes};

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ServiceRequest {
    /// The service: its project id as the app routes it.
    pub project: String,
    /// The service's title when asked (web's notification and rail rows name it).
    pub service: String,
    pub title: String,
    pub body: String,
    /// web's `scheduled_at` / `ends_at`, both or neither.
    pub starts_at: Option<Timestamp>,
    pub ends_at: Option<Timestamp>,
    /// The writer's invitation; `None` for the chat's creator.
    pub invite: Option<ActionHash>,
}

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct RequestStatus {
    pub request: ActionHash,
    /// `accepted`, `declined` or `cancelled`.
    pub status: String,
    /// A decline's optional reason.
    pub reason: String,
    pub invite: Option<ActionHash>,
}

fn window(r: &ServiceRequest) -> Result<Option<(i64, i64)>, String> {
    match (r.starts_at, r.ends_at) {
        (None, None) => Ok(None),
        (Some(s), Some(e)) => Ok(Some((s.as_micros(), e.as_micros()))),
        _ => Err("A timed request has a start and a finish".into()),
    }
}

/// web's request fields, in a direct chat, by someone in it.
pub fn validate_request(
    r: &ServiceRequest,
    author: &AgentPubKey,
    creator: &AgentPubKey,
    kind: ChatKind,
) -> ExternResult<ValidateCallbackResult> {
    if kind != ChatKind::Direct {
        return invalid("service requests go in a direct chat with the service's creator");
    }
    if let Err(reason) =
        window(r).and_then(|w| check_request(&r.project, &r.service, &r.title, &r.body, w))
    {
        return invalid(&reason);
    }
    if !member(author, r.invite.as_ref(), creator)? {
        return invalid("only people in the chat send requests in it");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// A known decision on a request in this chat, by whoever may make it.
pub fn validate_status(
    s: &RequestStatus,
    author: &AgentPubKey,
    creator: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_status(&s.status, &s.reason) {
        return invalid(&reason);
    }
    let record = must_get_valid_record(s.request.clone())?;
    if !is_type(record.action(), UnitEntryTypes::ServiceRequest)?
        || decode::<ServiceRequest>(&record)?.is_none()
    {
        return invalid("a request status settles a service request");
    }
    if !may_decide(&s.status, record.action().author() == author) {
        return invalid("the person asking cancels; the service's creator accepts or declines");
    }
    if !member(author, s.invite.as_ref(), creator)? {
        return invalid("only people in the chat settle its requests");
    }
    Ok(ValidateCallbackResult::Valid)
}
