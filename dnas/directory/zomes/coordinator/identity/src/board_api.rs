//! The API for moderators, the weekly active count and deactivation (ADRs 0095, 0088).

use hdk::prelude::*;

use crate::{active, board, deactivation, BoardView, BoardVote, WeeklyActive};

/// web's platform board (ADR 0095): moderators and candidates, with the quorum.
#[hdk_extern]
pub fn get_board(_: ()) -> ExternResult<BoardView> {
    board::view()
}

/// Volunteer as a moderator candidate (`true`) or withdraw (`false`).
#[hdk_extern]
pub fn volunteer_board(on: bool) -> ExternResult<ActionHash> {
    board::volunteer(on)
}

#[hdk_extern]
pub fn vote_board(input: BoardVote) -> ExternResult<ActionHash> {
    board::vote(input)
}

#[hdk_extern]
pub fn is_moderator(agent: AgentPubKey) -> ExternResult<bool> {
    board::is_moderator(&agent)
}

/// Write this week's `Active` link if the caller hasn't (ADR 0095). Returns whether it wrote.
#[hdk_extern]
pub fn record_active(_: ()) -> ExternResult<bool> {
    active::record()
}

/// Full members active in the last complete week before `at` (default now).
#[hdk_extern]
pub fn weekly_active(at: Option<Timestamp>) -> ExternResult<WeeklyActive> {
    active::count_at(at)
}

/// Deactivate the caller's account for good (ADR 0088). The UI confirms first.
#[hdk_extern]
pub fn deactivate_account(_: ()) -> ExternResult<Timestamp> {
    deactivation::deactivate()
}

#[hdk_extern]
pub fn deactivated_since(agent: AgentPubKey) -> ExternResult<Option<Timestamp>> {
    deactivation::since(&agent)
}

/// Write an `Active` link from `week`'s path exactly as asked, skipping `record_active`'s
/// check (see `submit_vouch`): tests prove a wrong week or a second link fails.
#[hdk_extern]
pub fn submit_active_link(week: i64) -> ExternResult<ActionHash> {
    let me = agent_info()?.agent_initial_pubkey;
    let path = identity_integrity::active_path(week).path_entry_hash()?;
    create_link(path, me, identity_integrity::LinkTypes::Active, ())
}
