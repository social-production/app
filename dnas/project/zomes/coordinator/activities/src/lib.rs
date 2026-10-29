//! Project DNA API for activities (ADR 0056): web's Activity phase. Members schedule
//! activities with roles, take or leave one role each, suggest people for roles, and rate an
//! activity they took part in once it has ended.

mod calls;
mod history;
mod rate;
mod reads;
mod view;
mod write;

use hdk::prelude::*;
use space_scope::{enter, touch, In};

pub use history::{CheckInInput, CompletionView};
pub use rate::RateInput;
pub use view::{ActivityView, RatingView, RoleView, SuggestionView};
pub use write::{NewActivity, NewRole, RoleInput, SuggestInput};

/// web's "Create activity" (Activity phase; members only).
#[hdk_extern]
pub fn create_activity(call: In<NewActivity>) -> ExternResult<ActionHash> {
    let input = enter(call)?;
    let out = write::create(input)?;
    touch()?;
    Ok(out)
}

#[hdk_extern]
pub fn list_activities(call: In<()>) -> ExternResult<Vec<ActivityView>> {
    enter(call)?;
    view::list()
}

/// Take a role (one per person per activity) before the activity ends.
#[hdk_extern]
pub fn take_role(call: In<RoleInput>) -> ExternResult<ActionHash> {
    let input = enter(call)?;
    let out = write::take(input)?;
    touch()?;
    Ok(out)
}

/// Leave your role in an activity before it ends.
#[hdk_extern]
pub fn leave_role(call: In<ActionHash>) -> ExternResult<()> {
    let activity = enter(call)?;
    write::leave(activity)?;
    touch()
}

/// Suggest someone else for a role; the latest suggestion per role stands.
#[hdk_extern]
pub fn suggest_role(call: In<SuggestInput>) -> ExternResult<ActionHash> {
    let input = enter(call)?;
    let out = write::suggest(input)?;
    touch()?;
    Ok(out)
}

/// Turn down a suggestion made for you.
#[hdk_extern]
pub fn decline_suggestion(call: In<ActionHash>) -> ExternResult<ActionHash> {
    let suggestion = enter(call)?;
    let out = write::decline(suggestion)?;
    touch()?;
    Ok(out)
}

/// Rate an activity you took part in (1–5, or clear it) once it has ended.
#[hdk_extern]
pub fn rate_activity(call: In<RateInput>) -> ExternResult<ActionHash> {
    let input = enter(call)?;
    let out = rate::rate(input)?;
    touch()?;
    Ok(out)
}

/// Say whether an activity you took part in happened, once it has ended (or clear it).
#[hdk_extern]
pub fn check_in(call: In<CheckInInput>) -> ExternResult<ActionHash> {
    let input = enter(call)?;
    let out = history::check_in(input)?;
    touch()?;
    Ok(out)
}
