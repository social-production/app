//! The caller's own writes in this chat, read from its source chain (visible before gossip).

use std::collections::BTreeSet;

use hdk::prelude::*;
use messages_integrity::{LinkTypes, UnitEntryTypes};

/// Every message the caller wrote in this chat, deleted or not.
pub fn messages() -> ExternResult<Vec<Record>> {
    let messages = EntryType::App(UnitEntryTypes::Message.try_into()?);
    query(
        ChainQueryFilter::new()
            .entry_type(messages)
            .action_type(ActionType::Create)
            .include_entries(true),
    )
}

/// The caller's messages it has deleted.
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

/// The caller's live chat links pointing at `target`.
pub fn live_links_to(target: &ActionHash) -> ExternResult<Vec<ActionHash>> {
    let scoped: ScopedLinkType = LinkTypes::Messages.try_into()?;
    let creates = query(ChainQueryFilter::new().action_type(ActionType::CreateLink))?;
    let removed: BTreeSet<ActionHash> =
        query(ChainQueryFilter::new().action_type(ActionType::DeleteLink))?
            .iter()
            .filter_map(|r| match &r.action().data {
                ActionData::DeleteLink(d) => Some(d.link_add_address.clone()),
                _ => None,
            })
            .collect();
    let wanted = AnyLinkableHash::from(target.clone());
    Ok(creates
        .iter()
        .filter(|r| match &r.action().data {
            ActionData::CreateLink(c) => {
                c.zome_index == scoped.zome_index
                    && c.link_type == scoped.zome_type
                    && c.target_address == wanted
            }
            _ => false,
        })
        .map(|r| r.action_address().clone())
        .filter(|hash| !removed.contains(hash))
        .collect())
}
