//! Links between projects (ADR 0042): each project holds and votes on its own half of a
//! request. The half proposed here names no source; the other project's half is a mirror
//! written by the proposer, naming this one as its source. Integrity can't see across
//! networks, so it checks each half on its own; the reader pairs them.

use hdi::prelude::*;
use sp_core::limits::{text_ok, LONG_TEXT_MAX};
use sp_governance::links::RELATIONSHIP_MAX;

use crate::{creator_in_seed, space_in_seed, typed_entry, EntryTypes, ProposalKind};

/// Another project: its directory listing (where the UI opens it) and its listing seed (a
/// private clone's network seed, or `<creator>.space.<id>` for a commons space, ADR 0065).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectRef {
    pub listing: ActionHash,
    pub network_seed: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LinkRequest {
    pub other: ProjectRef,
    pub relationship: String,
    pub summary: String,
    /// `Some(p)`: end the link that this project's create half `p` started (web's sever).
    pub sever: Option<ActionHash>,
    /// `Some(s)`: this half mirrors request `s` of the other project.
    pub source: Option<ActionHash>,
}

/// What a validator can check of one half, proposed in `space`.
pub fn check(link: &LinkRequest, space: &ActionHash) -> ExternResult<Result<(), String>> {
    if !text_ok(&link.relationship, RELATIONSHIP_MAX) || !text_ok(&link.summary, LONG_TEXT_MAX) {
        return Ok(Err("a link needs a label and a reason".into()));
    }
    if creator_in_seed(&link.other.network_seed).is_none() {
        return Ok(Err("a link names another project's network".into()));
    }
    // A commons space is named `<creator>.space.<id>` (ADR 0065); a private one by its seed.
    let itself = space_in_seed(&link.other.network_seed).as_ref() == Some(space)
        || dna_info()?.modifiers.network_seed == link.other.network_seed;
    if itself {
        return Ok(Err("a project can't link to itself".into()));
    }
    let Some(created) = &link.sever else {
        return Ok(Ok(()));
    };
    let ends_a_link = match typed_entry(&must_get_valid_record(created.clone())?)? {
        Some(EntryTypes::Proposal(p)) => matches!(
            p.kind,
            ProposalKind::Link(l) if l.sever.is_none() && l.other == link.other
        ),
        _ => false,
    };
    Ok(if ends_a_link {
        Ok(())
    } else {
        Err("a sever request ends a link to the same project".into())
    })
}
