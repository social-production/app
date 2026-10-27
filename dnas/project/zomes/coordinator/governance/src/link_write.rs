//! Filing one project's half of a link request (ADR 0042), with web-backend's checks from
//! `create_detail_link_request` and `create_detail_link_sever_request`. The UI files the
//! source half here, then mirrors it into the other project.

use hdk::prelude::*;
use project_integrity::{EntryTypes, LinkRequest, LinkTypes, Proposal, ProposalKind};
use sp_governance::links::{DEFAULT_RELATIONSHIP, RELATIONSHIP_MAX, SEVER_SUMMARY};

use crate::link_view::halves;
use crate::write::guest;
use crate::{calls, reads};

pub fn propose(link: LinkRequest) -> ExternResult<ActionHash> {
    let link = tidy(link)?;
    if dna_info()?.modifiers.network_seed == link.other.network_seed {
        return Err(guest("Cannot link a record to itself."));
    }
    let membership = calls::my_membership()?;
    let mine = halves()?;
    if let Some(source) = &link.source {
        // A mirror is filed once; filing it again returns the first.
        if let Some(h) = mine
            .iter()
            .find(|h| h.request.source.as_ref() == Some(source))
        {
            return Ok(h.id.clone());
        }
    } else {
        if membership.is_none() {
            return Err(guest("Only project members can perform this action."));
        }
        if let Some(created) = &link.sever {
            let link_create = mine.iter().any(|h| {
                &h.id == created && h.request.sever.is_none() && h.request.other == link.other
            });
            if !link_create {
                return Err(guest("Link not found."));
            }
            // web returns the sever request already open.
            if let Some(open) = mine
                .iter()
                .find(|h| h.request.sever.as_ref() == Some(created) && h.is_open())
            {
                return Ok(open.id.clone());
            }
        }
    }
    // In an organizer-controlled event an organizer's own half is decided at once (ADR 0061).
    let decides = link.source.is_none() && project_integrity::organized()?;
    let organizer = if decides {
        calls::my_organizer_role()?
            .ok_or_else(|| guest("Only the creator or co-organizers can link this event."))?
            .grant()
    } else {
        None
    };
    let hash = create_entry(EntryTypes::Proposal(Proposal {
        kind: ProposalKind::Link(link),
        membership,
        members: calls::member_snapshot()?,
        organizer,
    }))?;
    create_link(reads::anchor()?, hash.clone(), LinkTypes::Proposals, ())?;
    if decides {
        crate::write::decide_now(&hash)?;
    }
    Ok(hash)
}

/// web trims both texts, defaults the label to "Linked" and a sever's reason to its own.
fn tidy(mut link: LinkRequest) -> ExternResult<LinkRequest> {
    link.relationship = link.relationship.trim().to_string();
    if link.relationship.is_empty() {
        link.relationship = DEFAULT_RELATIONSHIP.into();
    }
    if link.relationship.chars().count() > RELATIONSHIP_MAX {
        return Err(guest("Keep the link label to 120 characters."));
    }
    link.summary = link.summary.trim().to_string();
    if link.summary.is_empty() {
        if link.sever.is_none() {
            return Err(guest("Say why these should be linked."));
        }
        link.summary = SEVER_SUMMARY.into();
    }
    Ok(link)
}
