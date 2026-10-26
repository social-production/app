//! Project DNA API for activities (ADR 0056): web's Activity phase. Members schedule
//! activities with roles, take or leave one role each, suggest people for roles, and rate an
//! activity they took part in once it has ended.

mod calls;
mod reads;
mod view;
mod write;

use hdk::prelude::*;

pub use view::{ActivityView, RatingView, RoleView, SuggestionView};
pub use write::{NewActivity, NewRole, RateInput, RoleInput, SuggestInput};

/// web's "Create activity" (Activity phase; members only).
#[hdk_extern]
pub fn create_activity(input: NewActivity) -> ExternResult<ActionHash> {
    write::create(input)
}

#[hdk_extern]
pub fn list_activities() -> ExternResult<Vec<ActivityView>> {
    view::list()
}

/// Take a role (one per person per activity) before the activity ends.
#[hdk_extern]
pub fn take_role(input: RoleInput) -> ExternResult<ActionHash> {
    write::take(input)
}

/// Leave your role in an activity before it ends.
#[hdk_extern]
pub fn leave_role(activity: ActionHash) -> ExternResult<()> {
    write::leave(activity)
}

/// Suggest someone else for a role; the latest suggestion per role stands.
#[hdk_extern]
pub fn suggest_role(input: SuggestInput) -> ExternResult<ActionHash> {
    write::suggest(input)
}

/// Turn down a suggestion made for you.
#[hdk_extern]
pub fn decline_suggestion(suggestion: ActionHash) -> ExternResult<ActionHash> {
    write::decline(suggestion)
}

/// Rate an activity you took part in (1–5, or clear it) once it has ended.
#[hdk_extern]
pub fn rate_activity(input: RateInput) -> ExternResult<ActionHash> {
    write::rate(input)
}
