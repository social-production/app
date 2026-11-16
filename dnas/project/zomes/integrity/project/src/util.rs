//! Small helpers every validation module uses.

use hdi::prelude::*;

use crate::EntryTypes;

pub(crate) fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(reason.to_string()))
}

/// Decode a record's entry using its declared entry type (never by guessing from bytes).
pub(crate) fn typed_entry(record: &Record) -> ExternResult<Option<EntryTypes>> {
    let Some(entry) = record.entry().as_option() else {
        return Ok(None);
    };
    let Some(EntryType::App(def)) = record.action().entry_type() else {
        return Ok(None);
    };
    EntryTypes::deserialize_from_type(def.zome_index, def.entry_index, entry)
}
