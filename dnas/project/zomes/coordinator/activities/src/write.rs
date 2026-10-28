//! Writes ported from web-backend `create_project_activity`, `commit_/uncommit_project_
//! activity_role`, `suggest_/decline_project_activity_role_suggestion` and
//! `upsert_/delete_project_activity_rating` (ADR 0056).

use hdk::prelude::*;
use project_integrity::{
    Activity, ActivityRating, ActivityRole, EntryTypes, LinkTypes, RoleSuggestion,
    SuggestionDecline,
};
use sp_activity::{check_activity, check_rating, roles_open, ActivitySpec, RoleSpec};

use crate::calls;
use crate::reads::{self, Found};
use crate::view;

pub fn guest(message: impl Into<String>) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(message.into()))
}

/// One role of a new activity, with web's optional suggested person.
#[derive(Serialize, Deserialize, Debug)]
pub struct NewRole {
    pub label: String,
    pub required: u32,
    pub maximum: Option<u32>,
    pub suggest: Option<AgentPubKey>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewActivity {
    pub title: String,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
    pub online: bool,
    pub place: String,
    pub meeting_url: Option<String>,
    pub note: String,
    pub plan_stage: Option<String>,
    pub roles: Vec<NewRole>,
}

fn member() -> ExternResult<ActionHash> {
    calls::my_membership()?.ok_or_else(|| guest("Join the project to take part in activities."))
}

/// web: members add activities in the Activity phase (the author only, in a personal service).
fn in_activity_phase() -> ExternResult<()> {
    let project = calls::glance()?;
    let personal = project
        .charter
        .as_ref()
        .is_some_and(|c| c.mode == "personal-service");
    let phase = if personal { "phase-1" } else { "phase-5" };
    if personal && project.creator != agent_info()?.agent_initial_pubkey {
        return Err(guest(
            "Only the person offering this service adds its activities.",
        ));
    }
    if project.phase == phase {
        Ok(())
    } else {
        Err(guest(
            "Activities can only be created during the activity phase",
        ))
    }
}

pub fn create(input: NewActivity) -> ExternResult<ActionHash> {
    let membership = member()?;
    in_activity_phase()?;
    if input.starts_at <= sys_time()? {
        return Err(guest("An activity has to start in the future."));
    }
    // web: only the creator or chosen organizers add activities to an organizer-controlled event.
    let organizer = if project_integrity::organized()? {
        calls::my_organizer_role()?
            .ok_or_else(|| {
                guest("Only the creator or chosen organizers can create activities for this event")
            })?
            .grant()
    } else {
        None
    };
    let activity = Activity {
        title: input.title.trim().to_string(),
        starts_at: input.starts_at,
        ends_at: input.ends_at,
        online: input.online,
        place: input.place.trim().to_string(),
        meeting_url: input
            .meeting_url
            .map(|u| u.trim().to_string())
            .filter(|u| !u.is_empty()),
        note: input.note.trim().to_string(),
        plan_stage: input.plan_stage.filter(|s| !s.is_empty()),
        roles: input
            .roles
            .iter()
            .map(|r| ActivityRole {
                label: r.label.trim().to_string(),
                required: r.required,
                maximum: r.maximum,
            })
            .collect(),
        membership: membership.clone(),
        organizer,
    };
    check_activity(&spec(&activity)).map_err(guest)?;
    let hash = create_entry(EntryTypes::Activity(activity))?;
    create_link(reads::anchor()?, hash.clone(), LinkTypes::Activities, ())?;
    let me = agent_info()?.agent_initial_pubkey;
    for (role, r) in input.roles.into_iter().enumerate() {
        if let Some(person) = r.suggest.filter(|p| *p != me) {
            suggest_as(&hash, role_index(role)?, person, membership.clone())?;
        }
    }
    Ok(hash)
}

pub fn spec(a: &Activity) -> ActivitySpec<'_> {
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

fn role_index(i: usize) -> ExternResult<u8> {
    u8::try_from(i).map_err(|_| guest("No such role."))
}

fn activity(hash: &ActionHash) -> ExternResult<Found<Activity>> {
    reads::found::<Activity>(hash.clone())?
        .ok_or_else(|| guest("This activity hasn't reached your node yet."))
}

fn open(a: &Activity) -> ExternResult<()> {
    if roles_open(a.ends_at.as_micros(), sys_time()?.as_micros()) {
        Ok(())
    } else {
        Err(guest("Roles are locked once an activity has ended."))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoleInput {
    pub activity: ActionHash,
    pub role: u8,
}

/// Take a role: one per person per activity, while it hasn't ended and isn't full.
pub fn take(input: RoleInput) -> ExternResult<ActionHash> {
    let membership = member()?;
    let found = activity(&input.activity)?;
    open(&found.entry)?;
    let Some(role) = found.entry.roles.get(usize::from(input.role)) else {
        return Err(guest("Role not found"));
    };
    let holders = view::holders(&input.activity, &found.entry)?;
    let me = agent_info()?.agent_initial_pubkey;
    if holders.iter().flatten().any(|a| *a == me) {
        return Err(guest("You already have a role in this activity."));
    }
    let filled = holders[usize::from(input.role)].len();
    if role.maximum.is_some_and(|m| filled >= m as usize) {
        return Err(guest("Role is already full"));
    }
    let mut tag = vec![input.role];
    tag.extend_from_slice(membership.get_raw_39());
    create_link(
        input.activity,
        me,
        LinkTypes::Assignments,
        LinkTag::new(tag),
    )
}

/// Leave whatever role you hold in `activity` (web's uncommit), before it ends.
pub fn leave(activity_hash: ActionHash) -> ExternResult<()> {
    open(&activity(&activity_hash)?.entry)?;
    let me = agent_info()?.agent_initial_pubkey;
    for take in reads::takes(&activity_hash)? {
        if take.agent == me {
            delete_link(take.link, GetOptions::default())?;
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SuggestInput {
    pub activity: ActionHash,
    pub role: u8,
    pub person: AgentPubKey,
}

pub fn suggest(input: SuggestInput) -> ExternResult<ActionHash> {
    let membership =
        calls::my_membership()?.ok_or_else(|| guest("Only project members can suggest a role"))?;
    if input.person == agent_info()?.agent_initial_pubkey {
        return Err(guest("Suggest someone else for this role"));
    }
    let found = activity(&input.activity)?;
    if usize::from(input.role) >= found.entry.roles.len() {
        return Err(guest("Role not found"));
    }
    suggest_as(&input.activity, input.role, input.person, membership)
}

fn suggest_as(
    activity: &ActionHash,
    role: u8,
    person: AgentPubKey,
    membership: ActionHash,
) -> ExternResult<ActionHash> {
    let hash = create_entry(EntryTypes::RoleSuggestion(RoleSuggestion {
        activity: activity.clone(),
        role,
        person,
        membership,
    }))?;
    create_link(
        activity.clone(),
        hash.clone(),
        LinkTypes::ActivityToSuggestions,
        (),
    )?;
    Ok(hash)
}

/// The suggested person turns the suggestion down.
pub fn decline(suggestion: ActionHash) -> ExternResult<ActionHash> {
    let found = reads::found::<RoleSuggestion>(suggestion.clone())?
        .ok_or_else(|| guest("This suggestion hasn't reached your node yet."))?;
    if found.entry.person != agent_info()?.agent_initial_pubkey {
        return Err(guest("Only the suggested user can decline"));
    }
    let hash = create_entry(EntryTypes::SuggestionDecline(SuggestionDecline {
        suggestion,
    }))?;
    create_link(
        found.entry.activity,
        hash.clone(),
        LinkTypes::ActivityToSuggestions,
        (),
    )?;
    Ok(hash)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RateInput {
    pub activity: ActionHash,
    /// 1–5, or `None` to remove your rating.
    pub stars: Option<u8>,
    pub comment: String,
}

/// Rate an activity you took part in once it has ended; the latest counts.
pub fn rate(input: RateInput) -> ExternResult<ActionHash> {
    let found = activity(&input.activity)?;
    let me = agent_info()?.agent_initial_pubkey;
    let Some(assignment) = view::my_take(&input.activity, &me)? else {
        return Err(guest("Only participants can rate activities"));
    };
    let comment = input.comment.trim().to_string();
    if let Some(stars) = input.stars {
        let now = sys_time()?.as_micros();
        check_rating(stars, &comment, found.entry.ends_at.as_micros(), now).map_err(guest)?;
    }
    let hash = create_entry(EntryTypes::ActivityRating(ActivityRating {
        activity: input.activity.clone(),
        stars: input.stars,
        comment,
        assignment,
    }))?;
    create_link(
        input.activity,
        hash.clone(),
        LinkTypes::ActivityToRatings,
        (),
    )?;
    Ok(hash)
}
