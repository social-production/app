//! Organizers of a private event (ADR 0061, 0073): the creator plus `Organizer` grants. Any
//! organizer may add one (the grant cites the granter's own record unless the granter is the
//! creator) or remove one: a removal is an `Organizers` link tagged `revoke` to the grant.
//! Integrity accepts any `Organizers` link, so every reader replays grants and removals in
//! time order and keeps only those written by someone who was an organizer at that moment.

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{audience, creator, EntryTypes, LinkTypes, Organizer, OrganizerRole};
use space_scope::{enter, In};

use crate::links;

/// The tag on an `Organizers` link that removes the grant it points at.
const REVOKE: &[u8] = b"revoke";

fn guest(message: &str) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(message.into()))
}

/// Grants in force (one per agent: the earliest), and when each removed grant ended.
#[derive(Default)]
pub struct Ledger {
    pub active: Vec<(ActionHash, Organizer)>,
    pub ended: BTreeMap<ActionHash, Timestamp>,
}

impl Ledger {
    fn organizer(&self, creator: Option<&AgentPubKey>, agent: &AgentPubKey) -> bool {
        creator == Some(agent) || self.active.iter().any(|(_, o)| &o.agent == agent)
    }
}

/// Replay every grant and removal this node can see, oldest first.
pub fn ledger() -> ExternResult<Ledger> {
    let creator = creator()?;
    let mut found = links(LinkTypes::Organizers)?;
    found.sort_by_key(|l| (l.timestamp, l.tag.0 == REVOKE));
    let mut ledger = Ledger::default();
    for link in found {
        let Some(hash) = link.target.into_action_hash() else {
            continue;
        };
        if link.tag.0 == REVOKE {
            if ledger.organizer(creator.as_ref(), &link.author) {
                if let Some(i) = ledger.active.iter().position(|(h, _)| *h == hash) {
                    ledger.active.remove(i);
                    ledger.ended.insert(hash, link.timestamp);
                }
            }
            continue;
        }
        if ledger.ended.contains_key(&hash) {
            continue;
        }
        let Some(record) = get(hash.clone(), GetOptions::default())? else {
            continue;
        };
        let Ok(Some(o)) = record.entry().to_app_option::<Organizer>() else {
            continue;
        };
        let granter = record.action().author();
        let fresh = ledger.active.iter().all(|(_, seen)| seen.agent != o.agent);
        if fresh && ledger.organizer(creator.as_ref(), granter) {
            ledger.active.push((hash, o));
        }
    }
    Ok(ledger)
}

/// The creator first, then co-organizers in the order they were added.
pub fn list() -> ExternResult<Vec<AgentPubKey>> {
    let mut out: Vec<AgentPubKey> = creator()?.into_iter().collect();
    for (_, o) in ledger()?.active {
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
    Ok(ledger()?
        .active
        .into_iter()
        .find(|(_, o)| &o.agent == agent)
        .map(|(hash, _)| OrganizerRole::Granted(hash)))
}

fn require_organizer(what: &str) -> ExternResult<OrganizerRole> {
    if !audience()?.is_private() {
        return Err(guest("Only private events have co-organizers."));
    }
    role_of(&agent_info()?.agent_initial_pubkey)?
        .ok_or_else(|| guest(&format!("Only organizers can {what} organizers.")))
}

/// Make `agent` an organizer (they still accept the invite to become a member).
pub fn add(agent: AgentPubKey) -> ExternResult<ActionHash> {
    let mine = require_organizer("add")?;
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
    create_link(
        space_scope::base()?,
        hash.clone(),
        LinkTypes::Organizers,
        (),
    )?;
    space_scope::touch()?;
    Ok(hash)
}

/// web's "Remove organizer": end `agent`'s grant (the creator's authority can't be removed).
pub fn remove(agent: AgentPubKey) -> ExternResult<()> {
    require_organizer("remove")?;
    if creator()?.as_ref() == Some(&agent) {
        return Err(guest(
            "Cannot revoke the event creator's organizer authority.",
        ));
    }
    let Some(OrganizerRole::Granted(grant)) = role_of(&agent)? else {
        return Ok(());
    };
    create_link(
        space_scope::base()?,
        grant,
        LinkTypes::Organizers,
        LinkTag::new(REVOKE),
    )?;
    space_scope::touch()
}

#[hdk_extern]
pub fn remove_organizer(call: In<AgentPubKey>) -> ExternResult<()> {
    let agent = enter(call)?;
    remove(agent)
}

/// Removed grants and when each ended: governance and activities leave out what a removed
/// organizer decided or added afterwards (ADR 0073).
#[hdk_extern]
pub fn organizer_ends(call: In<()>) -> ExternResult<Vec<(ActionHash, Timestamp)>> {
    enter(call)?;
    Ok(ledger()?.ended.into_iter().collect())
}
