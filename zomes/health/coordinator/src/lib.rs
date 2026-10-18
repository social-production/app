//! Health coordinator zome. Zome API used by the UI (`ui/src/lib/services/health.ts`)
//! and by the scenario tests (`tests/scenarios`).

use hdk::prelude::*;
use health_integrity::{EntryTypes, Heartbeat, LinkTypes};

const ANCHOR: &str = "all_heartbeats";

/// Cheapest possible round trip: proves the client reached a running cell.
#[hdk_extern]
pub fn ping(_: ()) -> ExternResult<String> {
    Ok("pong".to_string())
}

#[hdk_extern]
pub fn post_heartbeat(note: String) -> ExternResult<ActionHash> {
    let hash = create_entry(&EntryTypes::Heartbeat(Heartbeat { note }))?;
    let anchor = Path::from(ANCHOR).path_entry_hash()?;
    create_link(anchor, hash.clone(), LinkTypes::AllHeartbeats, ())?;
    Ok(hash)
}

/// Heartbeat action hashes this node can currently see, oldest first.
#[hdk_extern]
pub fn list_heartbeats(_: ()) -> ExternResult<Vec<ActionHash>> {
    let anchor = Path::from(ANCHOR).path_entry_hash()?;
    let mut links = get_links(
        LinkQuery::try_new(anchor, LinkTypes::AllHeartbeats)?,
        GetStrategy::default(),
    )?;
    links.sort_by_key(|link| link.timestamp);
    Ok(links
        .into_iter()
        .filter_map(|link| link.target.into_action_hash())
        .collect())
}

#[hdk_extern]
pub fn get_heartbeat(hash: ActionHash) -> ExternResult<Option<Record>> {
    get(hash, GetOptions::default())
}
