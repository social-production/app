//! Calls this zome makes to the project zome of the same cell.

use hdk::prelude::*;
use serde::de::DeserializeOwned;

fn project<T: DeserializeOwned + std::fmt::Debug>(func: &str) -> ExternResult<T> {
    let response = call(
        CallTargetCell::Local,
        ZomeName::from("project"),
        func.into(),
        None,
        space_scope::pass(())?,
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
    #[serde(default)]
    pub location_label: String,
    #[serde(default)]
    pub place: Option<project_integrity::Place>,
}

pub fn glance() -> ExternResult<Glance> {
    project::<Option<Glance>>("get_project")?
        .ok_or_else(|| wasm_error!(WasmErrorInner::Guest("This project isn't set up.".into())))
}
