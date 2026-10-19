//! Validation helpers shared by the identity rules: verdicts, typed decoding and reading
//! the author's own chain (`must_get_agent_activity`), which every validator can reproduce.

use hdi::prelude::*;

use crate::{EntryTypes, UnitEntryTypes};

pub(crate) fn same_author(
    a: &AgentPubKey,
    b: &AgentPubKey,
    reason: &str,
) -> ExternResult<ValidateCallbackResult> {
    if a == b {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid(reason)
    }
}

pub(crate) fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(reason.to_string()))
}

/// Decode a record's entry using its declared entry type (never by guessing from bytes).
pub(crate) fn typed_entry(record: &Record) -> ExternResult<Option<EntryTypes>> {
    let Some(entry) = record.entry().as_option() else {
        return Ok(None);
    };
    let Some(def) = record.action().entry_type().and_then(|t| match t {
        EntryType::App(def) => Some(def.clone()),
        _ => None,
    }) else {
        return Ok(None);
    };
    EntryTypes::deserialize_from_type(def.zome_index, def.entry_index, entry)
}

/// The action before a create (every create has one: genesis comes first).
pub(crate) fn prev_action(action: &TypedAction<CreateData>) -> ExternResult<ActionHash> {
    action.prev_action().cloned().ok_or_else(|| {
        wasm_error!(WasmErrorInner::Guest(
            "create without a previous action".into()
        ))
    })
}

/// Times of the author's `unit` creates in the `window_us` before this action, from their own chain.
pub(crate) fn earlier_creates(
    action: &TypedAction<CreateData>,
    unit: UnitEntryTypes,
    window_us: i64,
) -> ExternResult<Vec<i64>> {
    let since = Timestamp::from_micros(action.timestamp().as_micros() - window_us);
    let filter = ChainFilter::until_timestamp(prev_action(action)?, since);
    let entry_type = EntryType::App(unit.try_into()?);
    let activity = must_get_agent_activity(action.author().clone(), filter)?;
    Ok(activity
        .iter()
        .map(|a| a.action.hashed.content.clone())
        .filter(|a| a.entry_type() == Some(&entry_type) && matches!(a.data, ActionData::Create(_)))
        .map(|a| a.timestamp().as_micros())
        .collect())
}
