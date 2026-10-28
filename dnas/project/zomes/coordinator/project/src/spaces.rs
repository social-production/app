//! Which spaces to look at (ADR 0065): the caller's own, for the rail, notifications and
//! public chats; and the ones written to lately, for an index.

use hdk::prelude::*;
use project_integrity::{space_creator, LinkTypes};

use crate::own;

/// A space the caller is a member of, with its creator (the UI names it `<creator>.space.<id>`).
#[derive(Serialize, Deserialize, Debug)]
pub struct MySpace {
    pub space: ActionHash,
    pub creator: AgentPubKey,
}

/// Every space the caller is a member of in this cell, from its own chain (ADR 0065): the
/// commons' replacement for "the project clones on this node".
#[hdk_extern]
pub fn my_spaces(_: ()) -> ExternResult<Vec<MySpace>> {
    let mut out: Vec<MySpace> = Vec::new();
    for (_, link) in own::live_anywhere(LinkTypes::Members)? {
        let Some(space) = link.base_address.into_action_hash() else {
            continue;
        };
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
