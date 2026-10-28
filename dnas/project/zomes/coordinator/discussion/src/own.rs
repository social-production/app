//! The caller's own chat writes, read from its source chain (visible before gossip).

use std::collections::BTreeSet;

use discussion_integrity::{ChatMessage, LinkTypes, ReadMark, UnitEntryTypes};
use hdk::prelude::*;

use crate::anchor;

/// Every chat message the caller wrote in this space, deleted or not.
pub fn messages() -> ExternResult<Vec<Record>> {
    let space = anchor()?;
    let messages = EntryType::App(UnitEntryTypes::ChatMessage.try_into()?);
    let mine = query(
        ChainQueryFilter::new()
            .entry_type(messages)
            .action_type(ActionType::Create)
            .include_entries(true),
    )?;
    Ok(mine
        .into_iter()
        .filter(|r| {
            r.entry()
                .to_app_option::<ChatMessage>()
                .ok()
                .flatten()
                .is_some_and(|m| m.space == space)
        })
        .collect())
}

/// The spaces of every chat message the caller wrote, each once.
pub fn spaces() -> ExternResult<Vec<ActionHash>> {
    let messages = EntryType::App(UnitEntryTypes::ChatMessage.try_into()?);
    let mine = query(
        ChainQueryFilter::new()
            .entry_type(messages)
            .action_type(ActionType::Create)
            .include_entries(true),
    )?;
    let mut out: Vec<ActionHash> = Vec::new();
    for m in mine
        .iter()
        .filter_map(|r| r.entry().to_app_option::<ChatMessage>().ok().flatten())
    {
        if !out.contains(&m.space) {
            out.push(m.space);
        }
    }
    Ok(out)
}

/// How far the caller has read this space's chat.
pub fn read_at() -> ExternResult<Option<Timestamp>> {
    let space = anchor()?;
    let marks = query(
        ChainQueryFilter::new()
            .entry_type(EntryType::App(UnitEntryTypes::ReadMark.try_into()?))
            .include_entries(true),
    )?;
    Ok(marks
        .iter()
        .filter_map(|r| r.entry().to_app_option::<ReadMark>().ok().flatten())
        .filter(|m| m.space == space)
        .map(|m| m.at)
        .max())
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
    let scoped: ScopedLinkType = LinkTypes::ChatMessages.try_into()?;
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
