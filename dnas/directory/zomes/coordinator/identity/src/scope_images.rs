//! Pictures and covers for channels and communities (ADR 0109). An image is its own record,
//! content-addressed: the scope's charter, its passed edits and its listing name it by hash.

use hdk::prelude::*;
use identity_integrity::{EntryTypes, ScopeImage};

/// Store an image (or find the same one already stored) and return the hash charters cite.
#[hdk_extern]
pub fn put_scope_image(image: ScopeImage) -> ExternResult<EntryHash> {
    let hash = hash_entry(&image)?;
    if get(hash.clone(), GetOptions::local())?.is_none() {
        create_entry(&EntryTypes::ScopeImage(image))?;
    }
    Ok(hash)
}

/// The image a charter or listing names, if it can be found.
#[hdk_extern]
pub fn scope_image(hash: EntryHash) -> ExternResult<Option<ScopeImage>> {
    let Some(record) = get(hash, GetOptions::default())? else {
        return Ok(None);
    };
    record
        .entry()
        .to_app_option::<ScopeImage>()
        .map_err(|e| wasm_error!(e))
}
