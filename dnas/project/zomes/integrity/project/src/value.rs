//! Project values (ADR 0041): web's "Add value" in the Proposal phase and members' 1–10
//! importance votes. Integrity checks what every validator can reproduce (the label, the
//! range, membership); the Proposal-phase rule and duplicate labels are the coordinator's,
//! and reads keep the earliest of two same-named values.

use hdi::prelude::*;
use sp_core::limits::text_ok;
use sp_governance::values::{importance_ok, LABEL_MAX};

use crate::links::is_membership;
use crate::{invalid, EntryTypes};

/// "What should this achieve?"
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Value {
    pub label: String,
    /// The author's own `Members` link.
    pub membership: ActionHash,
}

/// One member's importance vote on a value; the latest counts (web lets members change it).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Importance {
    pub value: ActionHash,
    pub membership: ActionHash,
    pub importance: u8,
}

pub fn validate_value(
    v: &Value,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    if crate::organized()? {
        return invalid("organizer-controlled events don't use values");
    }
    if !text_ok(&v.label, LABEL_MAX) || v.label.trim() != v.label {
        return invalid("a value is 1–160 characters, trimmed");
    }
    // The membership chooses the value's space.
    if !is_membership(
        &v.membership,
        action.author(),
        Some(action.timestamp()),
        None,
    )? {
        return invalid("only members can add values");
    }
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_importance(
    i: &Importance,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    if crate::organized()? {
        return invalid("organizer-controlled events don't use values");
    }
    let Some(space) = value_space(&i.value)? else {
        return invalid("an importance vote must be on a value");
    };
    if !importance_ok(i.importance) {
        return invalid("importance is 1 to 10");
    }
    if !is_membership(
        &i.membership,
        action.author(),
        Some(action.timestamp()),
        Some(&space),
    )? {
        return invalid("only members can rate values");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// `hash` is a `Value` record.
pub fn is_value(hash: &ActionHash) -> ExternResult<bool> {
    Ok(value_space(hash)?.is_some())
}

/// The space of the `Value` record at `hash` (its author's membership's), or `None`.
fn value_space(hash: &ActionHash) -> ExternResult<Option<ActionHash>> {
    let record = must_get_valid_record(hash.clone())?;
    match crate::typed_entry(&record)? {
        Some(EntryTypes::Value(v)) => crate::links::membership_space(&v.membership),
        _ => Ok(None),
    }
}
