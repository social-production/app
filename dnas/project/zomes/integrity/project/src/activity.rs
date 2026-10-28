//! Activities and roles (ADR 0056): web's activity phase. An activity lists its roles; taking
//! one is an `Assignments` link (activity → yourself, tag = role index + your `Members` link),
//! so leaving deletes your own link. Integrity checks what every validator can reproduce
//! (membership, limits, the action's time against the activity's end); the Activity-phase rule
//! and "starts in the future" are the coordinator's, and who holds a role when two people take
//! the last place at once is the view's (`sp_activity::fill`).

use hdi::prelude::*;
use sp_activity::{check_activity, check_rating, roles_open, ActivitySpec, RoleSpec};

use crate::links::{is_membership, membership_space};
use crate::{invalid, EntryTypes};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ActivityRole {
    pub label: String,
    pub required: u32,
    pub maximum: Option<u32>,
}

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Activity {
    pub title: String,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
    pub online: bool,
    pub place: String,
    pub meeting_url: Option<String>,
    pub note: String,
    /// The approved plan stage this carries out, web's `linked_plan_phase_id`
    /// ("production:<stage>"); its label is resolved from the plans when read.
    pub plan_stage: Option<String>,
    pub roles: Vec<ActivityRole>,
    /// The author's own `Members` link.
    pub membership: ActionHash,
    /// The author's `Organizer` record in an organizer-controlled event (ADR 0061).
    #[serde(default)]
    pub organizer: Option<ActionHash>,
}

/// A member suggests someone else for a role (web's role suggestion); the latest per role
/// stands until the suggested person declines it.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct RoleSuggestion {
    pub activity: ActionHash,
    pub role: u8,
    pub person: AgentPubKey,
    pub membership: ActionHash,
}

/// The suggested person turns a suggestion down.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct SuggestionDecline {
    pub suggestion: ActionHash,
}

/// A participant's 1–5 rating once the activity has ended; the latest counts and `None`
/// removes it (web's delete).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ActivityRating {
    pub activity: ActionHash,
    pub stars: Option<u8>,
    pub comment: String,
    /// The author's `Assignments` link on this activity: only participants rate.
    pub assignment: ActionHash,
}

fn spec(a: &Activity) -> ActivitySpec<'_> {
    ActivitySpec {
        title: &a.title,
        starts_at: a.starts_at.as_micros(),
        ends_at: a.ends_at.as_micros(),
        online: a.online,
        place: &a.place,
        meeting_url: a.meeting_url.as_deref(),
        note: &a.note,
        plan_stage: a.plan_stage.as_deref(),
        roles: a
            .roles
            .iter()
            .map(|r| RoleSpec {
                label: &r.label,
                required: r.required,
                maximum: r.maximum,
            })
            .collect(),
    }
}

pub fn validate_activity(
    a: &Activity,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_activity(&spec(a)) {
        return invalid(&reason);
    }
    // The membership chooses the activity's space.
    if !is_membership(&a.membership, action.author(), Some(action.timestamp()), None)? {
        return invalid("only members can add activities");
    }
    let organizer = a.organizer.as_ref();
    if let Some(refused) =
        crate::organizer::require_in_organized(action.author(), organizer, "add activities")?
    {
        return Ok(refused);
    }
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_suggestion(
    s: &RoleSuggestion,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    let Some(activity) = activity_at(&s.activity)? else {
        return invalid("a suggestion is for an activity's role");
    };
    if usize::from(s.role) >= activity.roles.len() {
        return invalid("no such role");
    }
    if &s.person == action.author() {
        return invalid("suggest someone else for this role");
    }
    let space = membership_space(&activity.membership)?;
    if !is_membership(&s.membership, action.author(), Some(action.timestamp()), space.as_ref())? {
        return invalid("only project members can suggest a role");
    }
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_decline(
    d: &SuggestionDecline,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    let record = must_get_valid_record(d.suggestion.clone())?;
    match crate::typed_entry(&record)? {
        Some(EntryTypes::RoleSuggestion(s)) if &s.person == action.author() => {
            Ok(ValidateCallbackResult::Valid)
        }
        _ => invalid("only the suggested person can decline"),
    }
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
    if !is_own_assignment(&r.assignment, &r.activity, action.author())? {
        return invalid("only participants can rate activities");
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
    /// The author's `Assignments` link on this activity: only participants check in.
    pub assignment: ActionHash,
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
    if !is_own_assignment(&c.assignment, &c.activity, action.author())? {
        return invalid("only assigned participants can set participant completion");
    }
    Ok(ValidateCallbackResult::Valid)
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

/// Taking a role: activity → yourself, tag `[role]` + your `Members` link, before it ends.
pub fn validate_assignment(
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    if action.target_address.clone().into_agent_pub_key().as_ref() != Some(action.author()) {
        return invalid("people take roles as themselves");
    }
    let Some(activity_hash) = action.base_address.clone().into_action_hash() else {
        return invalid("a role belongs to an activity");
    };
    let Some(activity) = activity_at(&activity_hash)? else {
        return invalid("a role belongs to an activity");
    };
    let Some((&role, membership)) = action.tag.0.split_first() else {
        return invalid("an assignment names its role");
    };
    if usize::from(role) >= activity.roles.len() {
        return invalid("no such role");
    }
    let Ok(membership) = ActionHash::try_from_raw_39(membership.to_vec()) else {
        return invalid("an assignment cites its author's membership");
    };
    let space = membership_space(&activity.membership)?;
    if !is_membership(&membership, action.author(), Some(action.timestamp()), space.as_ref())? {
        return invalid("only members can take roles");
    }
    open_at(&activity, action.timestamp())
}

/// Leaving a role: only before the activity ends (web's `ensure_activity_roles_unlocked`).
pub fn validate_leave(
    base: &AnyLinkableHash,
    at: Timestamp,
) -> ExternResult<ValidateCallbackResult> {
    let activity = match base.clone().into_action_hash() {
        Some(hash) => activity_at(&hash)?,
        None => None,
    };
    match activity {
        Some(activity) => open_at(&activity, at),
        None => invalid("a role belongs to an activity"),
    }
}

fn open_at(activity: &Activity, at: Timestamp) -> ExternResult<ValidateCallbackResult> {
    if roles_open(activity.ends_at.as_micros(), at.as_micros()) {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid("roles are locked once an activity has ended")
    }
}

/// The `Activity` record at `hash`, or `None` when it is something else.
pub fn activity_at(hash: &ActionHash) -> ExternResult<Option<Activity>> {
    let record = must_get_valid_record(hash.clone())?;
    match crate::typed_entry(&record)? {
        Some(EntryTypes::Activity(a)) => Ok(Some(a)),
        _ => Ok(None),
    }
}
