//! Calls this zome makes to the project zome of the same cell.

use hdk::prelude::*;
use serde::de::DeserializeOwned;

fn project<T: DeserializeOwned + std::fmt::Debug>(func: &str) -> ExternResult<T> {
    let response = call(
        CallTargetCell::Local,
        ZomeName::from("project"),
        func.into(),
        None,
        (),
    )?;
    match response {
        ZomeCallResponse::Ok(io) => io.decode().map_err(|e| wasm_error!(e)),
        other => Err(wasm_error!(WasmErrorInner::Guest(format!(
            "project/{func}: {other:?}"
        )))),
    }
}

pub fn my_membership() -> ExternResult<Option<ActionHash>> {
    project("my_membership")
}

/// The parts of the project zome's `ProjectView` activities need.
#[derive(Deserialize, Debug)]
pub struct Glance {
    pub creator: AgentPubKey,
    pub charter: Option<CharterGlance>,
    pub phase: String,
    pub members: Vec<AgentPubKey>,
}

#[derive(Deserialize, Debug)]
pub struct CharterGlance {
    pub mode: String,
}

pub fn glance() -> ExternResult<Glance> {
    project::<Option<Glance>>("get_project")?
        .ok_or_else(|| wasm_error!(WasmErrorInner::Guest("This project isn't set up.".into())))
}
