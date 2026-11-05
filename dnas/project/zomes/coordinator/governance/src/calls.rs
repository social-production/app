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

/// Whether `agent` held Full standing before `at`, from the caller's own directory cell.
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
