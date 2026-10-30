//! The charter founds a space; its create hash is the space's id (ADR 0065). Passed edits
//! apply on top (ADR 0033).

use hdk::prelude::*;
use project_integrity::{creator, Charter, EntryTypes, LinkTypes};

/// Found a space: in the commons any number; in a private clone its one space, which the
/// "charter" anchor then points at. The founder joins it.
pub fn create(charter: Charter) -> ExternResult<ActionHash> {
    let private = creator()?.is_some();
    if private && space_scope::clone_space()?.is_some() {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "this private space already has a charter".into()
        )));
    }
    let hash = create_entry(EntryTypes::Charter(charter))?;
    if private {
        create_link(
            Path::from(space_scope::CHARTER).path_entry_hash()?,
            hash.clone(),
            LinkTypes::ProjectCharter,
            (),
        )?;
    }
    space_scope::set(hash.clone());
    crate::people::join(&[])?;
    space_scope::touch()?;
    Ok(hash)
}

/// The current space's charter as created, and when.
pub fn base() -> ExternResult<Option<(Charter, Timestamp)>> {
    let Some(record) = get(space_scope::current()?, GetOptions::default())? else {
        return Ok(None);
    };
    let charter = record
        .entry()
        .to_app_option::<Charter>()
        .map_err(|e| wasm_error!(e))?;
    Ok(charter.map(|c| (c, record.action().timestamp())))
}
