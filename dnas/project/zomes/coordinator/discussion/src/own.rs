//! The caller's own talk, read from its source chain (visible before gossip).

use std::collections::BTreeSet;

use discussion_integrity::{LinkTypes, UnitEntryTypes};
use hdk::prelude::*;

/// Every entry of `kind` the caller wrote, deleted or not.
pub fn entries(kind: UnitEntryTypes) -> ExternResult<Vec<Record>> {
    let wanted = EntryType::App(kind.try_into()?);
    query(
        ChainQueryFilter::new()
            .entry_type(wanted)
            .action_type(ActionType::Create)
            .include_entries(true),
    )
}

/// Every chat message the caller wrote, deleted or not.
pub fn messages() -> ExternResult<Vec<Record>> {
    entries(UnitEntryTypes::ChatMessage)
}

/// The caller's entries it has deleted.
pub fn deleted() -> ExternResult<BTreeSet<ActionHash>> {
    let deletes = query(ChainQueryFilter::new().action_type(ActionType::Delete))?;
    Ok(deletes
        .iter()
        .filter_map(|r| match &r.action().data {
            ActionData::Delete(d) => Some(d.deletes_address.clone()),
            _ => None,
        })
        .collect())
}

/// One of the caller's live links: the create-link action and what it says.
#[derive(Debug, Clone)]
pub struct OwnLink {
    pub hash: ActionHash,
    pub base: AnyLinkableHash,
    pub target: AnyLinkableHash,
    pub tag: LinkTag,
}

/// The caller's links of `link_type` it hasn't removed, oldest first.
pub fn live_links(link_type: LinkTypes) -> ExternResult<Vec<OwnLink>> {
    let scoped: ScopedLinkType = link_type.try_into()?;
    let creates = query(ChainQueryFilter::new().action_type(ActionType::CreateLink))?;
    let removed: BTreeSet<ActionHash> =
        query(ChainQueryFilter::new().action_type(ActionType::DeleteLink))?
            .iter()
            .filter_map(|r| match &r.action().data {
                ActionData::DeleteLink(d) => Some(d.link_add_address.clone()),
                _ => None,
            })
            .collect();
    Ok(creates
        .iter()
        .filter_map(|r| match &r.action().data {
            ActionData::CreateLink(c)
                if c.zome_index == scoped.zome_index && c.link_type == scoped.zome_type =>
            {
                Some(OwnLink {
                    hash: r.action_address().clone(),
                    base: c.base_address.clone(),
                    target: c.target_address.clone(),
                    tag: c.tag.clone(),
                })
            }
            _ => None,
        })
        .filter(|l| !removed.contains(&l.hash))
        .collect())
}

/// The caller's live chat links pointing at `target`.
pub fn live_links_to(target: &ActionHash) -> ExternResult<Vec<ActionHash>> {
    let wanted = AnyLinkableHash::from(target.clone());
    Ok(live_links(LinkTypes::ChatMessages)?
        .into_iter()
        .filter(|l| l.target == wanted)
        .map(|l| l.hash)
        .collect())
}
