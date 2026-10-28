//! Reading activity records off the DHT, plus the caller's own fresh writes (network reads lag
//! right after a write).

use hdk::prelude::*;
use project_integrity::{LinkTypes, UnitEntryTypes};

/// The current space's base: activities hang off it (ADR 0065).
pub fn anchor() -> ExternResult<AnyLinkableHash> {
    space_scope::base()
}

/// Whether the `Members` link at `membership` belongs to the current space (own records span
/// every space a person is in).
pub fn in_space(membership: &ActionHash) -> ExternResult<bool> {
    let Some(record) = get(membership.clone(), GetOptions::default())? else {
        return Ok(false);
    };
    let base = space_scope::base()?;
    Ok(matches!(&record.action().data, ActionData::CreateLink(l) if l.base_address == base))
}

#[derive(Debug)]
pub struct Found<T> {
    pub hash: ActionHash,
    pub author: AgentPubKey,
    pub at: Timestamp,
    pub entry: T,
}

pub fn found<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    hash: ActionHash,
) -> ExternResult<Option<Found<T>>> {
    found_of(hash, None)
}

/// The record at `hash` as a `T`; `None` when it is missing or (with `kind`) of another type,
/// since suggestions and declines share one link type.
fn found_of<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    hash: ActionHash,
    kind: Option<&EntryType>,
) -> ExternResult<Option<Found<T>>> {
    let Some(record) = get(hash.clone(), GetOptions::default())? else {
        return Ok(None);
    };
    if kind.is_some_and(|k| record.action().entry_type() != Some(k)) {
        return Ok(None);
    }
    let entry = record
        .entry()
        .to_app_option::<T>()
        .map_err(|e| wasm_error!(e))?;
    Ok(entry.map(|entry| Found {
        hash,
        author: record.action().author().clone(),
        at: record.action().timestamp(),
        entry,
    }))
}

/// Records linked from `base`, plus the caller's own records of that type, oldest first.
pub fn linked<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    base: impl Into<AnyLinkableHash>,
    link_type: LinkTypes,
    entry_type: UnitEntryTypes,
) -> ExternResult<Vec<Found<T>>> {
    let links = get_links(LinkQuery::try_new(base, link_type)?, GetStrategy::default())?;
    let mut hashes: Vec<ActionHash> = links
        .into_iter()
        .filter_map(|l| l.target.into_action_hash())
        .collect();
    let kind = EntryType::App(entry_type.try_into()?);
    let mine = query(
        ChainQueryFilter::new()
            .entry_type(kind.clone())
            .include_entries(true),
    )?;
    hashes.extend(mine.iter().map(|r| r.action_address().clone()));
    hashes.sort();
    hashes.dedup();
    let mut out: Vec<Found<T>> = hashes
        .into_iter()
        .filter_map(|h| found_of::<T>(h, Some(&kind)).transpose())
        .collect::<ExternResult<_>>()?;
    out.sort_by(|a, b| (a.at, &a.hash).cmp(&(b.at, &b.hash)));
    Ok(out)
}

/// One live role taken on `activity`: who, which role, when, and the link's hash.
#[derive(Debug, Clone)]
pub struct Take {
    pub agent: AgentPubKey,
    pub role: u8,
    pub at: Timestamp,
    pub link: ActionHash,
}

pub fn takes(activity: &ActionHash) -> ExternResult<Vec<Take>> {
    let query = LinkQuery::try_new(activity.clone(), LinkTypes::Assignments)?;
    let links = get_links(query, GetStrategy::default())?;
    Ok(links
        .into_iter()
        .filter_map(|l| {
            let role = *l.tag.0.first()?;
            Some(Take {
                agent: l.author,
                role,
                at: l.timestamp,
                link: l.create_link_hash,
            })
        })
        .collect())
}
