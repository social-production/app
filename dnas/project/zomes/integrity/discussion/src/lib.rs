//! Talk inside one project (ADR 0038). M4a: the project chat. As on web, anyone signed in may
//! write in a project's chat (the membrane is open, ADR 0006), so nothing here reads membership.
//! Messages are never edited (web's project chat has no edit); only their author deletes one.
//! Photos and files ride along since ADR 0056 (rules from `sp_attachment`, as ADR 0049), and
//! a private read mark keeps "unread" per person rather than per device (ADR 0044).

use hdi::prelude::*;
use sp_attachment::{check_attachment, check_body, check_chunk};
use sp_core::chat::check_message;

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ChatMessage {
    pub body: String,
    /// The message this one answers (web's `parent_id`), shown as a quote.
    pub reply_to: Option<ActionHash>,
    /// Photos and files (web's `message_attachments`); the body may then be empty.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// One photo or file: its description and the chunks holding its bytes, in order.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Attachment {
    pub kind: String,
    pub name: String,
    pub mime: String,
    pub size: u32,
    pub chunks: Vec<ActionHash>,
}

/// Part of an attachment's bytes (at most `sp_attachment::CHUNK_MAX`), written by the
/// message's author before the message.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Chunk {
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

/// How far the author has read this project's chat (private: only they see it).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ReadMark {
    pub at: Timestamp,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    ChatMessage(ChatMessage),
    Chunk(Chunk),
    #[entry_type(visibility = "private")]
    ReadMark(ReadMark),
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
        FlatOp::CreateEntry(OpEntry::CreateEntry { app_entry, action })
        | FlatOp::CreateRecord(OpRecord::CreateEntry { app_entry, action }) => match app_entry {
            EntryTypes::ChatMessage(m) => validate_message(&m, action.author()),
            EntryTypes::Chunk(c) => match check_chunk(c.bytes.len()) {
                Ok(()) => Ok(ValidateCallbackResult::Valid),
                Err(reason) => invalid(&reason),
            },
            EntryTypes::ReadMark(_) => Ok(ValidateCallbackResult::Valid),
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

fn validate_message(m: &ChatMessage, author: &AgentPubKey) -> ExternResult<ValidateCallbackResult> {
    let text = if m.attachments.is_empty() {
        check_message(&m.body)
    } else {
        check_body(&m.body, m.attachments.len())
    };
    if let Err(reason) = text {
        return invalid(&reason);
    }
    for a in &m.attachments {
        let size = usize::try_from(a.size).unwrap_or(usize::MAX);
        if let Err(reason) = check_attachment(&a.kind, &a.name, &a.mime, size, a.chunks.len()) {
            return invalid(&reason);
        }
        for chunk in &a.chunks {
            let signed = must_get_action(chunk.clone())?;
            if !is_type(signed.action(), UnitEntryTypes::Chunk)?
                || signed.action().author() != author
            {
                return invalid("a message's files are parts its author wrote");
            }
        }
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
    is_type(action, UnitEntryTypes::ChatMessage)
}

/// The action creates an entry of this zome's `kind`.
pub fn is_type(action: &Action, kind: UnitEntryTypes) -> ExternResult<bool> {
    let scoped: ScopedEntryDefIndex = kind.try_into()?;
    Ok(matches!(
        (action.action_type(), action.entry_type()),
        (ActionType::Create, Some(EntryType::App(def)))
            if def.zome_index == scoped.zome_index && def.entry_index == scoped.zome_type
    ))
}

fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(reason.to_string()))
}
