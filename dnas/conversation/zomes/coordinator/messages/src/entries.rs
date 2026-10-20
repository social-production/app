//! Reading one kind of chat entry: the caller's own chain (seen before gossip) plus what the
//! network lists under an anchor, each once, oldest first.

use std::collections::BTreeSet;

use hdk::prelude::*;
use messages_integrity::{is_type, LinkTypes, UnitEntryTypes};

use crate::reads;

/// One entry with who wrote it and when.
#[derive(Clone, Debug)]
pub struct Found<T> {
    pub hash: ActionHash,
    pub author: AgentPubKey,
    pub at: Timestamp,
    pub entry: T,
}

pub fn path(name: &str) -> ExternResult<EntryHash> {
    Path::from(name).path_entry_hash()
}

fn own_hashes(unit: UnitEntryTypes) -> ExternResult<Vec<ActionHash>> {
    Ok(query(
        ChainQueryFilter::new()
            .entry_type(EntryType::App(unit.try_into()?))
            .action_type(ActionType::Create),
    )?
    .iter()
    .map(|r| r.action_address().clone())
    .collect())
}

/// Every `unit` entry listed by `link` on the `anchor` path, plus the caller's own.
pub fn all<T>(unit: UnitEntryTypes, link: LinkTypes, anchor: &str) -> ExternResult<Vec<Found<T>>>
where
    T: TryFrom<SerializedBytes, Error = SerializedBytesError>,
{
    let mut hashes = own_hashes(unit)?;
    let links = get_links(
        LinkQuery::try_new(path(anchor)?, link)?,
        GetStrategy::default(),
    )?;
    hashes.extend(
        links
            .into_iter()
            .filter_map(|l| l.target.into_action_hash()),
    );
    let unique: Vec<ActionHash> = hashes
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    decoded(reads::records(&unique)?.into_iter().flatten(), unit)
}

/// The caller's own private `unit` entries, oldest first.
pub fn private<T>(unit: UnitEntryTypes) -> ExternResult<Vec<Found<T>>>
where
    T: TryFrom<SerializedBytes, Error = SerializedBytesError>,
{
    let records = query(
        ChainQueryFilter::new()
            .entry_type(EntryType::App(unit.try_into()?))
            .include_entries(true),
    )?;
    decoded(records.into_iter(), unit)
}

fn decoded<T>(
    records: impl Iterator<Item = Record>,
    unit: UnitEntryTypes,
) -> ExternResult<Vec<Found<T>>>
where
    T: TryFrom<SerializedBytes, Error = SerializedBytesError>,
{
    let mut out = Vec::new();
    for record in records {
        if !is_type(record.action(), unit)? {
            continue;
        }
        if let Some(entry) = record
            .entry()
            .to_app_option::<T>()
            .map_err(|e| wasm_error!(e))?
        {
            out.push(Found {
                hash: record.action_address().clone(),
                author: record.action().author().clone(),
                at: record.action().timestamp(),
                entry,
            });
        }
    }
    out.sort_by(|a, b| (a.at, &a.hash).cmp(&(b.at, &b.hash)));
    Ok(out)
}
