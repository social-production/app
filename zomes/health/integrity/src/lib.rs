//! Health integrity zome, included in every DNA. A `Heartbeat` is an immutable note
//! linked from a global anchor; the platform check (M0) and network monitoring use it
//! to prove authoring, validation and gossip work between two nodes.

use hdi::prelude::*;
use sp_core::limits::{text_ok, NOTE_MAX};

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Heartbeat {
    pub note: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    Heartbeat(Heartbeat),
}

#[derive(Serialize, Deserialize)]
#[hdk_link_types]
pub enum LinkTypes {
    AllHeartbeats,
}

#[hdk_extern]
pub fn genesis_self_check(_data: GenesisSelfCheckData) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::CreateEntry(OpEntry::CreateEntry { app_entry, .. })
        | FlatOp::CreateRecord(OpRecord::CreateEntry { app_entry, .. }) => {
            validate_create(&app_entry)
        }
        FlatOp::CreateEntry(OpEntry::UpdateEntry { .. })
        | FlatOp::Update(OpUpdate::Entry { .. })
        | FlatOp::CreateRecord(OpRecord::UpdateEntry { .. } | OpRecord::DeleteEntry { .. })
        | FlatOp::Delete(_) => invalid("heartbeats are immutable"),
        FlatOp::Link(OpLink::DeleteLink { .. })
        | FlatOp::CreateRecord(OpRecord::DeleteLink { .. }) => {
            invalid("heartbeat links are permanent")
        }
        _ => Ok(ValidateCallbackResult::Valid),
    }
}

fn validate_create(entry: &EntryTypes) -> ExternResult<ValidateCallbackResult> {
    match entry {
        EntryTypes::Heartbeat(beat) if text_ok(&beat.note, NOTE_MAX) => {
            Ok(ValidateCallbackResult::Valid)
        }
        EntryTypes::Heartbeat(_) => invalid("heartbeat note must be 1-280 characters"),
    }
}

fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(reason.to_string()))
}
