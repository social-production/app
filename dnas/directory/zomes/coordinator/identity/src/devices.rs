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

/// Agents paired with `agent` where both sides have written their half.
pub fn confirmed(agent: &AgentPubKey) -> ExternResult<Vec<AgentPubKey>> {
    let mut halves: Vec<(AgentPubKey, DeviceLink)> = Vec::new();
    for hash in action_targets(links(agent.clone(), LinkTypes::AgentToDeviceLinks)?) {
        let Some(record) = live(hash)? else { continue };
        if let Some(link) = decode::<DeviceLink>(&record)? {
            halves.push((record.action().author().clone(), link));
        }
    }
    let mut paired = Vec::new();
    for (author, link) in &halves {
        let other = if &link.primary == agent {
            &link.device
        } else {
            &link.primary
        };
        let mirrored = halves
            .iter()
            .any(|(a, l)| a != author && l.primary == link.primary && l.device == link.device);
        if mirrored && !paired.contains(other) && other != agent {
            paired.push(other.clone());
        }
    }
    Ok(paired)
}
