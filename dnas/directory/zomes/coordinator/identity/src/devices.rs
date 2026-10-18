//! Device pairing: each of the two agents writes its own `DeviceLink`; a pairing is
//! confirmed once both halves exist.

use hdk::prelude::*;
use identity_integrity::{DeviceLink, EntryTypes, LinkTypes};

use crate::guest_error;
use crate::records::{action_targets, decode, links, live};

pub fn link(link: DeviceLink) -> ExternResult<ActionHash> {
    let me = agent_info()?.agent_initial_pubkey;
    let other = if me == link.primary {
        link.device.clone()
    } else if me == link.device {
        link.primary.clone()
    } else {
        return Err(guest_error("You can only pair your own device."));
    };
    let hash = create_entry(&EntryTypes::DeviceLink(link))?;
    create_link(me, hash.clone(), LinkTypes::AgentToDeviceLinks, ())?;
    create_link(other, hash.clone(), LinkTypes::AgentToDeviceLinks, ())?;
    Ok(hash)
}

/// Both halves of every pairing involving `agent` (written by different agents).
fn confirmed_links(agent: &AgentPubKey) -> ExternResult<Vec<DeviceLink>> {
    let mut halves: Vec<(AgentPubKey, DeviceLink)> = Vec::new();
    for hash in action_targets(links(agent.clone(), LinkTypes::AgentToDeviceLinks)?) {
        let Some(record) = live(hash)? else { continue };
        if let Some(link) = decode::<DeviceLink>(&record)? {
            halves.push((record.action().author().clone(), link));
        }
    }
    let mut confirmed: Vec<DeviceLink> = Vec::new();
    for (author, link) in &halves {
        let mirrored = halves.iter().any(|(a, l)| a != author && l == link);
        if mirrored && !confirmed.contains(link) {
            confirmed.push(link.clone());
        }
    }
    Ok(confirmed)
}

/// Agents paired with `agent` (either direction).
pub fn confirmed(agent: &AgentPubKey) -> ExternResult<Vec<AgentPubKey>> {
    Ok(confirmed_links(agent)?
        .into_iter()
        .map(|l| {
            if &l.primary == agent {
                l.device
            } else {
                l.primary
            }
        })
        .filter(|other| other != agent)
        .collect())
}

/// Which person `agent` belongs to: its primary (if it is a paired device) and its devices.
#[derive(Serialize, Deserialize, Debug)]
pub struct Pairing {
    pub primary: Option<AgentPubKey>,
    pub devices: Vec<AgentPubKey>,
}

pub fn pairing(agent: &AgentPubKey) -> ExternResult<Pairing> {
    let links = confirmed_links(agent)?;
    let primary = links
        .iter()
        .find(|l| &l.device == agent)
        .map(|l| l.primary.clone());
    let devices = links
        .into_iter()
        .filter(|l| &l.primary == agent)
        .map(|l| l.device)
        .collect();
    Ok(Pairing { primary, devices })
}

/// Pairings someone else started with `me` that I have not confirmed yet.
pub fn requests() -> ExternResult<Vec<DeviceLink>> {
    let me = agent_info()?.agent_initial_pubkey;
    let mut mine: Vec<DeviceLink> = Vec::new();
    let mut theirs: Vec<DeviceLink> = Vec::new();
    for hash in action_targets(links(me.clone(), LinkTypes::AgentToDeviceLinks)?) {
        let Some(record) = live(hash)? else { continue };
        let Some(link) = decode::<DeviceLink>(&record)? else {
            continue;
        };
        if record.action().author() == &me {
            mine.push(link);
        } else if !theirs.contains(&link) {
            theirs.push(link);
        }
    }
    Ok(theirs.into_iter().filter(|l| !mine.contains(l)).collect())
}
