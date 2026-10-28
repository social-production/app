//! Ratings, ported from web-backend `upsert_/delete_project_activity_rating` (ADR 0056).

use hdk::prelude::*;
use project_integrity::{ActivityRating, EntryTypes, LinkTypes};
use sp_activity::check_rating;

use crate::view;
use crate::write::{activity, guest};

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
