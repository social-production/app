//! A personal service's weekly hours and bookings (web's `project_service_availability_rules`
//! and the booking an accepted timed request becomes; ADR 0077). Only the service's creator
//! lists hours or books; a rule is shown while its `Availability` link stands, so removing
//! it deletes the link. A booking names the request (a record in the private direct chat,
//! ADR 0076) and the person who asked, who may check in and rate it once it has ended.

use hdi::prelude::*;
use sp_activity::availability::{check_booking, check_rule};

use crate::{charter_at, invalid, typed_entry, EntryTypes};

/// web's weekly hours: a weekday (0 = Monday), minutes after midnight in `zone`.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct AvailabilityRule {
    /// The service: its charter's create hash.
    pub space: ActionHash,
    pub weekday: u8,
    pub start: u16,
    pub end: u16,
    pub zone: String,
    pub note: String,
}

/// The request an activity books (web's `linked_request_id`) and who asked.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Booking {
    pub request: ActionHash,
    pub requester: AgentPubKey,
}

/// The space is a personal service that `author` created.
fn creator_of_personal(space: &ActionHash, author: &AgentPubKey) -> ExternResult<bool> {
    Ok(charter_at(space)?
        .is_some_and(|(charter, creator)| charter.mode == "personal-service" && &creator == author))
}

pub fn validate_rule(
    r: &AvailabilityRule,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_rule(r.weekday, r.start, r.end, &r.zone, &r.note) {
        return invalid(&reason);
    }
    if !creator_of_personal(&r.space, action.author())? {
        return invalid("Only the service creator can edit availability");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// Space → its creator's rule.
pub fn validate_link(action: &TypedAction<CreateLinkData>) -> ExternResult<ValidateCallbackResult> {
    let Some(target) = action.target_address.clone().into_action_hash() else {
        return invalid("an availability link points at a rule");
    };
    let record = must_get_valid_record(target)?;
    match typed_entry(&record)? {
        Some(EntryTypes::AvailabilityRule(r))
            if record.action().author() == action.author()
                && action.base_address == AnyLinkableHash::from(r.space.clone()) =>
        {
            Ok(ValidateCallbackResult::Valid)
        }
        _ => invalid("a service lists its creator's own rules"),
    }
}

/// A booking is for someone else's request. In a personal service it is the creator's, with
/// web's one Provider role; in a collective service any member books it (the activity's
/// membership is checked by its caller) with whatever roles the request needs (ADR 0089).
pub fn validate_booking(
    activity: &crate::Activity,
    booking: &Booking,
    space: Option<&ActionHash>,
    author: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if &booking.requester == author {
        return invalid("a booking is for someone else's request");
    }
    let Some(space) = space else {
        return invalid("only services book requests");
    };
    match charter_at(space)? {
        // A proposal may have become a collective service (ADR 0059); the coordinator
        // checks its type now, since validation cannot replay its decisions.
        Some((charter, _)) if crate::applied::may_be(&charter.mode, "collective-service") => {
            Ok(ValidateCallbackResult::Valid)
        }
        Some((charter, creator)) if charter.mode == "personal-service" && &creator == author => {
            let roles: Vec<_> = activity
                .roles
                .iter()
                .map(|r| sp_activity::RoleSpec {
                    label: &r.label,
                    required: r.required,
                    maximum: r.maximum,
                    description: &r.description,
                })
                .collect();
            match check_booking(&roles) {
                Ok(()) => Ok(ValidateCallbackResult::Valid),
                Err(reason) => invalid(&reason),
            }
        }
        _ => invalid(
            "only a personal service's creator or a collective service's members book requests",
        ),
    }
}

/// Who may check in on or rate an activity without holding a role: a booking's requester.
pub fn is_requester(activity: &crate::Activity, author: &AgentPubKey) -> bool {
    activity
        .booking
        .as_ref()
        .is_some_and(|b| &b.requester == author)
}
