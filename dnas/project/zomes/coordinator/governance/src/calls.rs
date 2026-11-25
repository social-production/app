//! Calls this zome makes: the project zome of the same cell (membership, signals, charter)
//! and the agent's directory cell (standing, ADR 0033).

use hdk::prelude::*;
use project_integrity::Charter;
use serde::de::DeserializeOwned;

fn decode<T: DeserializeOwned + std::fmt::Debug>(
    response: ZomeCallResponse,
    what: &str,
) -> ExternResult<T> {
    match response {
        ZomeCallResponse::Ok(io) => io.decode().map_err(|e| wasm_error!(e)),
        other => Err(wasm_error!(WasmErrorInner::Guest(format!(
            "{what}: {other:?}"
        )))),
    }
}

fn project<T: DeserializeOwned + std::fmt::Debug>(func: &str) -> ExternResult<T> {
    let response = call(
        CallTargetCell::Local,
        ZomeName::from("project"),
        func.into(),
        None,
        space_scope::pass(())?,
    )?;
    decode(response, func)
}

pub fn my_membership() -> ExternResult<Option<ActionHash>> {
    project("my_membership")
}

pub fn member_snapshot() -> ExternResult<Vec<ActionHash>> {
    project("member_snapshot")
}

/// What makes the caller an organizer of this event, if anything (ADR 0061).
pub fn my_organizer_role() -> ExternResult<Option<project_integrity::OrganizerRole>> {
    project("my_organizer_role")
}

/// Removed organizer grants and when each ended (ADR 0073).
pub fn organizer_ends() -> ExternResult<std::collections::BTreeMap<ActionHash, Timestamp>> {
    let ends: Vec<(ActionHash, Timestamp)> = project("organizer_ends")?;
    Ok(ends.into_iter().collect())
}

/// Whether something cites a grant that had already been removed when it was written.
pub fn lapsed(
    ends: &std::collections::BTreeMap<ActionHash, Timestamp>,
    grant: Option<&ActionHash>,
    at: Timestamp,
) -> bool {
    grant
        .and_then(|g| ends.get(g))
        .is_some_and(|end| *end <= at)
}

pub fn base_charter() -> ExternResult<Option<Charter>> {
    project("base_charter")
}

/// Who founded the space this call is for (its charter's author).
pub fn base_creator() -> ExternResult<Option<AgentPubKey>> {
    let space = space_scope::current()?;
    Ok(get(space, GetOptions::default())?.map(|r| r.action().author().clone()))
}

#[derive(Deserialize, Debug)]
pub struct SignalCounts {
    pub demand: u32,
    pub opposition: u32,
}

pub fn signal_counts() -> ExternResult<SignalCounts> {
    project("signal_counts")
}

#[derive(Serialize, Debug)]
struct StandingAt {
    agent: AgentPubKey,
    at: Timestamp,
}

/// Whether `agent`'s votes count at `at` (not found to be a bot then; ADR 0118), from the
/// caller's own directory cell.
pub fn full_before(agent: &AgentPubKey, at: Timestamp) -> ExternResult<bool> {
    let query = StandingAt {
        agent: agent.clone(),
        at,
    };
    let response = call(
        CallTargetCell::OtherRole("directory".into()),
        ZomeName::from("identity"),
        "full_standing_before".into(),
        None,
        query,
    )?;
    decode(response, "identity/full_standing_before")
}

/// Whether `agent` was a platform moderator at `at`, from the caller's directory cell
/// (ADR 0095): a #platform software project's merges count from moderators too.
pub fn moderator_at(agent: &AgentPubKey, at: Timestamp) -> ExternResult<bool> {
    let query = StandingAt {
        agent: agent.clone(),
        at,
    };
    let response = call(
        CallTargetCell::OtherRole("directory".into()),
        ZomeName::from("identity"),
        "moderator_at".into(),
        None,
        query,
    )?;
    decode(response, "identity/moderator_at")
}

#[derive(Deserialize, Debug)]
struct WeeklyActive {
    count: u32,
}

/// People (not bots; ADR 0118) active in the seven days before `at` (ADR 0121), from the
/// caller's directory cell (ADR 0095): a #platform project's population (ADR 0109).
pub fn weekly_active(at: Timestamp) -> ExternResult<u32> {
    let response = call(
        CallTargetCell::OtherRole("directory".into()),
        ZomeName::from("identity"),
        "weekly_active".into(),
        None,
        Some(at),
    )?;
    decode::<WeeklyActive>(response, "identity/weekly_active").map(|w| w.count)
}

#[derive(Serialize, Debug)]
struct Among {
    agents: Vec<AgentPubKey>,
    at: Timestamp,
}

/// Which of `agents` were active (not as bots) in the seven days before `at` (ADR 0121).
pub fn active_among(agents: Vec<AgentPubKey>, at: Timestamp) -> ExternResult<Vec<AgentPubKey>> {
    if agents.is_empty() {
        return Ok(agents);
    }
    // Without a directory cell (a lone project network) everyone counts, as before ADR 0121.
    let response = call(
        CallTargetCell::OtherRole("directory".into()),
        ZomeName::from("identity"),
        "active_among".into(),
        None,
        Among {
            agents: agents.clone(),
            at,
        },
    );
    match response {
        Ok(ok @ ZomeCallResponse::Ok(_)) => decode(ok, "identity/active_among"),
        _ => Ok(agents),
    }
}
