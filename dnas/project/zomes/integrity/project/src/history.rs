//! Past activity (web's history): participants' 1–5 ratings and check-ins once an activity
//! has ended. Participants cite their `Assignments` link; a booking's requester cites none
//! (ADR 0077).

use hdi::prelude::*;
use sp_activity::{check_rating, roles_open};

use crate::activity::{activity_at, Activity};
use crate::invalid;

/// A participant's 1–5 rating once the activity has ended; the latest counts and `None`
/// removes it (web's delete).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ActivityRating {
    pub activity: ActionHash,
    pub stars: Option<u8>,
    pub comment: String,
    /// The author's `Assignments` link on this activity: only participants rate, and a
    /// booking's requester, who cites none (ADR 0077).
    pub assignment: Option<ActionHash>,
}

pub fn validate_rating(
    r: &ActivityRating,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    let Some(activity) = activity_at(&r.activity)? else {
        return invalid("a rating is for an activity");
    };
    if let Some(stars) = r.stars {
        let at = action.timestamp().as_micros();
        if let Err(reason) = check_rating(stars, &r.comment, activity.ends_at.as_micros(), at) {
            return invalid(&reason);
        }
    }
    if !takes_part(
        r.assignment.as_ref(),
        &r.activity,
        &activity,
        action.author(),
    )? {
        return invalid("Only participants or the requester can rate activities");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// A participant's check-in once the activity has ended (web's history completion): it
/// happened (`Some(true)`), it didn't (`Some(false)`), or cleared (`None`); the latest counts.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ActivityCompletion {
    pub activity: ActionHash,
    pub done: Option<bool>,
    /// The author's `Assignments` link on this activity: only participants check in, and a
    /// booking's requester, who cites none (ADR 0077).
    pub assignment: Option<ActionHash>,
}

pub fn validate_completion(
    c: &ActivityCompletion,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    let Some(activity) = activity_at(&c.activity)? else {
        return invalid("a check-in is for an activity");
    };
    if roles_open(activity.ends_at.as_micros(), action.timestamp().as_micros()) {
        return invalid("an activity is checked in once it has ended");
    }
    if !takes_part(
        c.assignment.as_ref(),
        &c.activity,
        &activity,
        action.author(),
    )? {
        return invalid("only assigned participants can set participant completion");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// The author held a role (cites their own assignment), or asked for this booking.
fn takes_part(
    assignment: Option<&ActionHash>,
    hash: &ActionHash,
    activity: &Activity,
    author: &AgentPubKey,
) -> ExternResult<bool> {
    match assignment {
        Some(link) => is_own_assignment(link, hash, author),
        None => Ok(crate::availability::is_requester(activity, author)),
    }
}

/// `hash` is an `Assignments` link `author` made on `activity` (deleted later or not).
fn is_own_assignment(
    hash: &ActionHash,
    activity: &ActionHash,
    author: &AgentPubKey,
) -> ExternResult<bool> {
    let taken = must_get_action(hash.clone())?;
    let ActionData::CreateLink(link) = &taken.action().data else {
        return Ok(false);
    };
    let assignments: ScopedLinkType = crate::LinkTypes::Assignments.try_into()?;
    let is_assignment =
        link.zome_index == assignments.zome_index && link.link_type == assignments.zome_type;
    let base = link.base_address.clone().into_action_hash();
    Ok(is_assignment && taken.action().author() == author && base.as_ref() == Some(activity))
}
