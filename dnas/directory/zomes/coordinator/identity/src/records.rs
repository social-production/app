//! Small read helpers shared by the identity modules.

use hdk::prelude::*;
use identity_integrity::LinkTypes;

pub fn links(base: impl Into<AnyLinkableHash>, link_type: LinkTypes) -> ExternResult<Vec<Link>> {
    get_links(LinkQuery::try_new(base, link_type)?, GetStrategy::default())
}

pub fn action_targets(links: Vec<Link>) -> Vec<ActionHash> {
    links
        .into_iter()
        .filter_map(|l| l.target.into_action_hash())
        .collect()
}

/// The newest version of an updatable record (by update timestamp), unless it was deleted.
pub fn latest(original: ActionHash) -> ExternResult<Option<Record>> {
    let Some(Details::Record(details)) = get_details(original, GetOptions::default())? else {
        return Ok(None);
    };
    if !details.deletes.is_empty() {
        return Ok(None);
    }
    let newest = details
        .updates
        .iter()
        .max_by_key(|u| u.action().timestamp())
        .map(|u| u.action_address().clone());
    match newest {
        Some(hash) => get(hash, GetOptions::default()),
        None => Ok(Some(details.record)),
    }
}

/// A record that exists and has not been deleted.
pub fn live(hash: ActionHash) -> ExternResult<Option<Record>> {
    let Some(Details::Record(details)) = get_details(hash, GetOptions::default())? else {
        return Ok(None);
    };
    Ok(details.deletes.is_empty().then_some(details.record))
}

pub fn decode<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    record: &Record,
) -> ExternResult<Option<T>> {
    record
        .entry()
        .to_app_option::<T>()
        .map_err(|e| wasm_error!(e))
}

/// The newest record of `unit` on the caller's own source chain (creates and updates).
/// Own writes are visible here immediately, before the network has integrated them.
pub fn own_latest(unit: identity_integrity::UnitEntryTypes) -> ExternResult<Option<Record>> {
    let filter = ChainQueryFilter::new()
        .entry_type(EntryType::App(unit.try_into()?))
        .include_entries(true);
    Ok(query(filter)?
        .into_iter()
        .max_by_key(|r| r.action().action_seq()))
}
