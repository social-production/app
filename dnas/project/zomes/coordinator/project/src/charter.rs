//! The charter: one original (linked from the "charter" anchor) plus the creator's updates.

use hdk::prelude::*;
use project_integrity::{Charter, EntryTypes, LinkTypes, UnitEntryTypes};

use crate::{anchor, links, CHARTER};

pub fn create(charter: Charter) -> ExternResult<ActionHash> {
    if original()?.is_some() {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "this project already has a charter".into()
        )));
    }
    let hash = create_entry(EntryTypes::Charter(charter))?;
    create_link(
        anchor(CHARTER)?,
        hash.clone(),
        LinkTypes::ProjectCharter,
        (),
    )?;
    crate::join(())?;
    Ok(hash)
}

pub fn update(charter: Charter) -> ExternResult<ActionHash> {
    let Some(original) = original()? else {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "this project has no charter yet".into()
        )));
    };
    update_entry(original, EntryTypes::Charter(charter))
}

/// The current charter and the times it was written (original first), newest version wins.
pub fn current() -> ExternResult<(Option<Charter>, Vec<Timestamp>)> {
    let Some(original) = original()? else {
        return Ok((None, Vec::new()));
    };
    let Some(Details::Record(details)) = get_details(original, GetOptions::default())? else {
        return Ok((None, Vec::new()));
    };
    let mut versions: Vec<Record> = details
        .updates
        .iter()
        .filter_map(|u| {
            get(u.action_address().clone(), GetOptions::default())
                .ok()
                .flatten()
        })
        .collect();
    versions.push(details.record);
    versions.sort_by_key(|r| r.action().timestamp());
    let times = versions.iter().map(|r| r.action().timestamp()).collect();
    let newest = versions
        .last()
        .map(|r| r.entry().to_app_option::<Charter>())
        .transpose()
        .map_err(|e| wasm_error!(e))?
        .flatten();
    Ok((newest, times))
}

/// The first charter: own chain first (fresh writes), then the network.
fn original() -> ExternResult<Option<ActionHash>> {
    let charters = EntryType::App(UnitEntryTypes::Charter.try_into()?);
    let own = query(
        ChainQueryFilter::new()
            .entry_type(charters)
            .action_type(ActionType::Create),
    )?;
    if let Some(first) = own.first() {
        return Ok(Some(first.action_address().clone()));
    }
    let mut found = links(CHARTER, LinkTypes::ProjectCharter)?;
    found.sort_by_key(|l| l.timestamp);
    Ok(found.into_iter().find_map(|l| l.target.into_action_hash()))
}
