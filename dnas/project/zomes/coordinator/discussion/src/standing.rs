//! Who counts on reports (ADR 0071): people with Full standing at the time of their vote, as
//! the directory says (the same rule as proposals, ADR 0033).

use hdk::prelude::*;

use crate::guest;

#[derive(Serialize, Debug)]
struct StandingAt {
    agent: AgentPubKey,
    at: Timestamp,
}

pub fn full_before(agent: &AgentPubKey, at: Timestamp) -> ExternResult<bool> {
    let query = StandingAt {
        agent: agent.clone(),
        at,
    };
    match call(
        CallTargetCell::OtherRole("directory".into()),
        ZomeName::from("identity"),
        "full_standing_before".into(),
        None,
        query,
    )? {
        ZomeCallResponse::Ok(io) => io.decode().map_err(|e| wasm_error!(e)),
        other => Err(guest(&format!("identity/full_standing_before: {other:?}"))),
    }
}
