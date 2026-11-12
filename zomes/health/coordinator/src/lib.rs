//! Health coordinator zome. Zome API used by the UI (`ui/src/lib/services/health.ts`)
//! and by the scenario tests (`tests/scenarios`).

use hdk::prelude::*;
use health_integrity::{EntryTypes, Heartbeat, LinkTypes};

const ANCHOR: &str = "all_heartbeats";
/// Desktops in "Help the network" mode announce their bootstrap server here (ADR 0094).
const SERVERS: &str = "bootstrap_servers";
/// An announcement older than this is not listed: a helping desktop re-announces on start.
const SERVER_FRESH_US: i64 = 14 * 24 * 3600 * 1_000_000;
/// An announcement is a short JSON note (`{"bootstrap":…,"relay":…}`), kept in the link tag.
const SERVER_NOTE_MAX: usize = 280;

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

/// Announce this node's bootstrap server: a link from the servers anchor to the author, the
/// note in its tag (no entry, so nothing to fetch when listing). The integrity zome already
/// accepts any heartbeat link, so this needs no network reset (ADR 0094).
#[hdk_extern]
pub fn announce_server(note: String) -> ExternResult<ActionHash> {
    if note.is_empty() || note.len() > SERVER_NOTE_MAX {
        return Err(wasm_error!(WasmErrorInner::Guest(format!(
            "an announcement must be 1-{SERVER_NOTE_MAX} bytes"
        ))));
    }
    let anchor = Path::from(SERVERS).path_entry_hash()?;
    let me = agent_info()?.agent_initial_pubkey;
    create_link(anchor, me, LinkTypes::AllHeartbeats, LinkTag::new(note))
}

/// An announced server as listed: the note, who announced it and when.
#[derive(Serialize, Deserialize, Debug)]
pub struct Announced {
    pub note: String,
    pub author: AgentPubKey,
    pub at: Timestamp,
}

/// Announcements from the last two weeks, newest first. Readers check each note themselves.
#[hdk_extern]
pub fn list_servers(_: ()) -> ExternResult<Vec<Announced>> {
    let anchor = Path::from(SERVERS).path_entry_hash()?;
    let links = get_links(
        LinkQuery::try_new(anchor, LinkTypes::AllHeartbeats)?,
        GetStrategy::default(),
    )?;
    let since = sys_time()?.as_micros() - SERVER_FRESH_US;
    let mut fresh: Vec<Announced> = links
        .into_iter()
        .filter(|link| link.timestamp.as_micros() >= since)
        .filter_map(|link| {
            let note = String::from_utf8(link.tag.into_inner()).ok()?;
            Some(Announced {
                note,
                author: link.author,
                at: link.timestamp,
            })
        })
        .collect();
    fresh.sort_by_key(|a| std::cmp::Reverse(a.at));
    Ok(fresh)
}
