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

/// Whether `agent` was a moderator (or in grace) at `at`: a merge recorded on a #platform
/// software project counts when its author was (ADR 0095).
#[hdk_extern]
pub fn moderator_at(q: crate::StandingAt) -> ExternResult<bool> {
    board::moderator_at(&q.agent, q.at)
}

/// Write an `Active` link unless the caller wrote one in the last hour (ADRs 0095, 0121).
/// Returns whether it wrote.
#[hdk_extern]
pub fn record_active(_: ()) -> ExternResult<bool> {
    active::record()
}

/// People (not bots) active in the seven days before `at` (default now; ADR 0121).
#[hdk_extern]
pub fn weekly_active(at: Option<Timestamp>) -> ExternResult<WeeklyActive> {
    active::count_at(at)
}

/// Which of `agents` were active (not as bots) in the seven days before `at`: a vote's
/// population is a space's members who were (ADR 0121, web's `weekly_active_project_members`).
#[hdk_extern]
pub fn active_among(q: active::Among) -> ExternResult<Vec<AgentPubKey>> {
    active::among(q)
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

/// Write an `Active` link from `day`'s path exactly as asked, skipping `record_active`'s
/// check (see `submit_vouch`): tests prove a wrong day or a second link within the hour fails.
#[hdk_extern]
pub fn submit_active_link(day: i64) -> ExternResult<ActionHash> {
    let me = agent_info()?.agent_initial_pubkey;
    let path = identity_integrity::active_path(day).path_entry_hash()?;
    create_link(path, me, identity_integrity::LinkTypes::Active, ())
}
