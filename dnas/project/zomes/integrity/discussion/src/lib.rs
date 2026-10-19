//! Talk inside one project (ADR 0038). M4a: the project chat. As on web, anyone signed in may
//! write in a project's chat (the membrane is open, ADR 0006), so nothing here reads membership.
//! Messages are never edited (web's project chat has no edit); only their author deletes one.

use hdi::prelude::*;
use sp_core::chat::check_message;

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ChatMessage {
    pub body: String,
    /// The message this one answers (web's `parent_id`), shown as a quote.
    pub reply_to: Option<ActionHash>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    ChatMessage(ChatMessage),
}

#[derive(Serialize, Deserialize)]
#[hdk_link_types]
pub enum LinkTypes {
    /// A chat anchor → a message, linked by the message's author. The coordinator chooses the
    /// anchor (one "chat" path today), so bucketing it by time later is a coordinator change.
    ChatMessages,
}

#[hdk_extern]
pub fn genesis_self_check(_data: GenesisSelfCheckData) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::CreateEntry(OpEntry::CreateEntry { app_entry, .. })
        | FlatOp::CreateRecord(OpRecord::CreateEntry { app_entry, .. }) => match app_entry {
            EntryTypes::ChatMessage(m) => validate_message(&m),
        },
        FlatOp::CreateEntry(OpEntry::UpdateEntry { .. })
        | FlatOp::CreateRecord(OpRecord::UpdateEntry { .. })
        | FlatOp::Update(_) => invalid("chat messages are never edited"),
        FlatOp::CreateRecord(OpRecord::DeleteEntry { action })
        | FlatOp::Delete(OpDelete { action }) => {
            let original = must_get_action(action.deletes_address.clone())?;
            if original.action().author() == action.author() {
                Ok(ValidateCallbackResult::Valid)
            } else {
                invalid("only a message's author can delete it")
            }
        }
        FlatOp::Link(OpLink::CreateLink { action, .. })
        | FlatOp::CreateRecord(OpRecord::CreateLink { action, .. }) => validate_link(&action),
        FlatOp::Link(OpLink::DeleteLink {
            original_action,
            action,
            ..
        }) => {
            if original_action.author() == action.author() {
                Ok(ValidateCallbackResult::Valid)
            } else {
                invalid("only the link's author can remove it")
            }
        }
        _ => Ok(ValidateCallbackResult::Valid),
    }
}

fn validate_message(m: &ChatMessage) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_message(&m.body) {
        return invalid(&reason);
    }
    if let Some(parent) = &m.reply_to {
        let record = must_get_valid_record(parent.clone())?;
        if !is_message(record.action())? {
            return invalid("a reply answers a chat message");
        }
    }
    Ok(ValidateCallbackResult::Valid)
}

/// Only a message's author lists it, so deleting their link takes it out of the chat for good.
fn validate_link(action: &TypedAction<CreateLinkData>) -> ExternResult<ValidateCallbackResult> {
    let Some(target) = action.target_address.clone().into_action_hash() else {
        return invalid("a chat link points at a message");
    };
    let signed = must_get_action(target)?;
    if is_message(signed.action())? && signed.action().author() == action.author() {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid("people list only their own messages in the chat")
    }
}

/// The action creates a `ChatMessage` of this zome (never decided by guessing from bytes).
pub fn is_message(action: &Action) -> ExternResult<bool> {
    let scoped: ScopedEntryDefIndex = UnitEntryTypes::ChatMessage.try_into()?;
    Ok(matches!(
        (action.action_type(), action.entry_type()),
        (ActionType::Create, Some(EntryType::App(def)))
            if def.zome_index == scoped.zome_index && def.entry_index == scoped.zome_type
    ))
}

fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(reason.to_string()))
}
