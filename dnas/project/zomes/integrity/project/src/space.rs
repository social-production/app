//! Spaces (ADR 0065): a project, event or other public space is a charter record in the
//! commons (the project DNA's base cell, no network seed); a private space is a clone whose
//! seed names its creator and holds that one space. A space's id is its charter's create
//! action hash, and every record of the space hangs off it, so validators read the creator and
//! kind from the charter itself rather than from the network seed.

use hdi::prelude::*;
use holo_hash::{ActionHashB64, AgentPubKeyB64};

use crate::{typed_entry, Charter, EntryTypes};

/// The clone's creator, from the network seed; `None` in the commons (the base cell).
pub fn creator() -> ExternResult<Option<AgentPubKey>> {
    Ok(creator_in_seed(&dna_info()?.modifiers.network_seed))
}

/// `<creator key>.<nonce>` → the creator key.
pub fn creator_in_seed(seed: &str) -> Option<AgentPubKey> {
    let (key, nonce) = seed.split_once('.')?;
    if nonce.is_empty() {
        return None;
    }
    AgentPubKeyB64::from_b64_str(key).ok().map(Into::into)
}

/// The word in a directory listing's seed that marks a commons space: `<creator>.space.<id>`.
pub const SPACE_SEED_WORD: &str = "space";

/// A commons space named by a directory listing's seed (`<creator>.space.<charter hash>`).
pub fn space_in_seed(seed: &str) -> Option<ActionHash> {
    let mut parts = seed.splitn(3, '.');
    let (_creator, word, id) = (parts.next()?, parts.next()?, parts.next()?);
    if word != SPACE_SEED_WORD {
        return None;
    }
    ActionHashB64::from_b64_str(id).ok().map(Into::into)
}

/// The charter at `space` and who wrote it (the space's creator), or `None` when `space` isn't
/// a charter.
pub fn charter_at(space: &ActionHash) -> ExternResult<Option<(Charter, AgentPubKey)>> {
    let record = must_get_valid_record(space.clone())?;
    let author = record.action().author().clone();
    match typed_entry(&record)? {
        Some(EntryTypes::Charter(c)) => Ok(Some((c, author))),
        _ => Ok(None),
    }
}

/// The creator of the space at `space` (`None` when it isn't a space).
pub fn space_creator(space: &ActionHash) -> ExternResult<Option<AgentPubKey>> {
    Ok(charter_at(space)?.map(|(_, author)| author))
}

/// `base` is a space (a charter's create action).
pub fn is_space(base: &AnyLinkableHash) -> ExternResult<bool> {
    match base.clone().into_action_hash() {
        Some(hash) => Ok(charter_at(&hash)?.is_some()),
        None => Ok(false),
    }
}
