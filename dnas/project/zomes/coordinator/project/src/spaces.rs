//! Which spaces to look at (ADR 0065): the caller's own, for the rail, notifications and
//! public chats; and the ones written to lately, for an index.

use hdk::prelude::*;
use project_integrity::{space_creator, LinkTypes};

use crate::own;

/// Spaces whose chat the caller wrote in (the discussion zome of this cell).
fn chat_spaces() -> ExternResult<Vec<ActionHash>> {
    match call(
        CallTargetCell::Local,
        ZomeName::from("discussion"),
        "my_chat_spaces".into(),
        None,
        (),
    )? {
        ZomeCallResponse::Ok(io) => io.decode().map_err(|e| wasm_error!(e)),
        other => Err(wasm_error!(WasmErrorInner::Guest(format!(
            "discussion/my_chat_spaces: {other:?}"
        )))),
    }
}

/// One of the caller's spaces, with its creator (the UI names it `<creator>.space.<id>`).
#[derive(Serialize, Deserialize, Debug)]
pub struct MySpace {
    pub space: ActionHash,
    pub creator: AgentPubKey,
}

/// Every space the caller is a member of or wrote in the chat of, from its own chain (ADR
/// 0065): the commons' replacement for "the project clones on this node".
#[hdk_extern]
pub fn my_spaces(_: ()) -> ExternResult<Vec<MySpace>> {
    let members = own::live_anywhere(LinkTypes::Members)?
        .into_iter()
        .filter_map(|(_, link)| link.base_address.into_action_hash());
    let mut out: Vec<MySpace> = Vec::new();
    for space in members.chain(chat_spaces()?) {
        if out.iter().any(|s| s.space == space) {
            continue;
        }
        if let Some(creator) = space_creator(&space)? {
            out.push(MySpace { space, creator });
        }
    }
    Ok(out)
}

/// Spaces written to in any of `hours` (hours since the Unix epoch): what an index re-reads.
#[hdk_extern]
pub fn active_spaces(hours: Vec<i64>) -> ExternResult<Vec<ActionHash>> {
    let mut out: Vec<ActionHash> = Vec::new();
    for hour in hours {
        for space in space_scope::active_in(hour)? {
            if !out.contains(&space) {
                out.push(space);
            }
        }
    }
    Ok(out)
}
