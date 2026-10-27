//! Organizers of a private event (ADR 0061): the creator plus `Organizer` grants. Any organizer
//! may add one; the grant cites the granter's own record unless the granter is the creator.

use hdk::prelude::*;
use project_integrity::{audience, creator, EntryTypes, LinkTypes, Organizer, OrganizerRole};

use crate::{anchor, links};

pub(crate) const ORGANIZERS: &str = "organizers";

fn guest(message: &str) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(message.into()))
}

/// Every grant this node can see, with its record hash (one per agent: the earliest).
pub fn grants() -> ExternResult<Vec<(ActionHash, Organizer)>> {
    let mut out: Vec<(ActionHash, Organizer)> = Vec::new();
    let mut found = links(ORGANIZERS, LinkTypes::Organizers)?;
    found.sort_by_key(|l| l.timestamp);
    for link in found {
        let Some(hash) = link.target.into_action_hash() else {
            continue;
        };
        let Some(record) = get(hash.clone(), GetOptions::default())? else {
            continue;
        };
        let Ok(Some(o)) = record.entry().to_app_option::<Organizer>() else {
            continue;
        };
        if out.iter().all(|(_, seen)| seen.agent != o.agent) {
            out.push((hash, o));
        }
    }
    Ok(out)
}

/// The creator first, then co-organizers in the order they were added.
pub fn list() -> ExternResult<Vec<AgentPubKey>> {
    let mut out: Vec<AgentPubKey> = creator()?.into_iter().collect();
    for (_, o) in grants()? {
        if !out.contains(&o.agent) {
            out.push(o.agent);
        }
    }
    Ok(out)
}

/// What makes `agent` an organizer, if anything.
pub fn role_of(agent: &AgentPubKey) -> ExternResult<Option<OrganizerRole>> {
    if creator()?.as_ref() == Some(agent) {
        return Ok(Some(OrganizerRole::Creator));
    }
    Ok(grants()?
        .into_iter()
        .find(|(_, o)| &o.agent == agent)
        .map(|(hash, _)| OrganizerRole::Granted(hash)))
}

/// Make `agent` an organizer (they still accept the invite to become a member).
pub fn add(agent: AgentPubKey) -> ExternResult<ActionHash> {
    if !audience()?.is_private() {
        return Err(guest("Only private events have co-organizers."));
    }
    let me = agent_info()?.agent_initial_pubkey;
    let Some(mine) = role_of(&me)? else {
        return Err(guest("Only organizers can add organizers."));
    };
    if let Some(OrganizerRole::Granted(existing)) = role_of(&agent)? {
        return Ok(existing);
    }
    if creator()?.as_ref() == Some(&agent) {
        return Err(guest("The creator is already an organizer."));
    }
    let hash = create_entry(EntryTypes::Organizer(Organizer {
        agent,
        grant: mine.grant(),
    }))?;
    create_link(anchor(ORGANIZERS)?, hash.clone(), LinkTypes::Organizers, ())?;
    Ok(hash)
}
