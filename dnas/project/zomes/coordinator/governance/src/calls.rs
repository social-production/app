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
        (),
    )?;
    decode(response, func)
}

pub fn my_membership() -> ExternResult<Option<ActionHash>> {
    project("my_membership")
}

pub fn member_snapshot() -> ExternResult<Vec<ActionHash>> {
    project("member_snapshot")
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
