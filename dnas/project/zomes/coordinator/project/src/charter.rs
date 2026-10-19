//! The charter as created (linked from the "charter" anchor); passed edits apply on top (ADR 0033).

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
    crate::people::join()?;
    Ok(hash)
}

/// The charter as created, and when.
pub fn base() -> ExternResult<Option<(Charter, Timestamp)>> {
    let Some(original) = original()? else {
        return Ok(None);
    };
    let Some(record) = get(original, GetOptions::default())? else {
        return Ok(None);
    };
    let charter = record
        .entry()
        .to_app_option::<Charter>()
        .map_err(|e| wasm_error!(e))?;
    Ok(charter.map(|c| (c, record.action().timestamp())))
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
