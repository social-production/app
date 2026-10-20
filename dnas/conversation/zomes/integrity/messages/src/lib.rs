//! Direct messages and group chats (ADR 0043). Each chat is a clone of this DNA seeded
//! `<creator key>.<kind>.<nonce>`: its creator invites people, and only the creator and the
//! people they invited write there. The uncloned base cell is everyone's inbox: an `Inbox` link
//! on a person's key carries an invitation (the chat's seed), encrypted to them.

mod chat;

pub use chat::is_type;

use hdi::prelude::*;
use sp_core::conversation::{chat_seed, ChatKind};

/// The creator brings `agent` into the chat (web's `conversation_members` row).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Invite {
    pub agent: AgentPubKey,
}

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Message {
    pub body: String,
    /// The message this one answers (web's `reply_to_id`), shown as a quote.
    pub reply_to: Option<ActionHash>,
    /// The author's invitation; `None` for the chat's creator.
    pub invite: Option<ActionHash>,
}

/// A group's name; the newest one by the creator counts (web's rename).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Title {
    pub title: String,
}

/// How far the author has read (web's `last_read_at`). Private: never leaves their chain.
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
    Invite(Invite),
    Message(Message),
    Title(Title),
    #[entry_type(visibility = "private")]
    ReadMark(ReadMark),
}

#[derive(Serialize, Deserialize)]
#[hdk_link_types]
pub enum LinkTypes {
    /// Chat anchor → an invitation, by the creator: who is in the chat.
    Invites,
    /// Chat anchor → a message, by its author.
    Messages,
    /// Chat anchor → a group name, by the creator.
    Titles,
    /// Inbox network only: a person's key → the sender's key, the tag an invitation encrypted
    /// to that person.
    Inbox,
}

/// Which network this cell is: one chat (its creator and kind) or the inbox.
#[derive(Debug)]
pub enum Network {
    Chat(AgentPubKey, ChatKind),
    Inbox,
}

pub fn network() -> ExternResult<Network> {
    let seed = dna_info()?.modifiers.network_seed;
    Ok(match chat_seed(&seed) {
        Some((creator, kind)) => match AgentPubKeyB64::from_b64_str(creator) {
            Ok(key) => Network::Chat(key.into(), kind),
            Err(_) => Network::Inbox,
        },
        None => Network::Inbox,
    })
}

#[hdk_extern]
pub fn genesis_self_check(_data: GenesisSelfCheckData) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {
        FlatOp::CreateEntry(OpEntry::CreateEntry { app_entry, action })
        | FlatOp::CreateRecord(OpRecord::CreateEntry { app_entry, action }) => {
            chat::validate_entry(&app_entry, action.author())
        }
        FlatOp::CreateRecord(OpRecord::CreatePrivateEntry { action, .. }) => {
            chat::validate_private(action.author())
        }
        FlatOp::CreateEntry(OpEntry::UpdateEntry { .. })
        | FlatOp::CreateRecord(OpRecord::UpdateEntry { .. })
        | FlatOp::Update(_) => invalid("chat entries are never edited"),
        FlatOp::CreateRecord(OpRecord::DeleteEntry { action })
        | FlatOp::Delete(OpDelete { action }) => {
            let original = must_get_action(action.deletes_address.clone())?;
            let own = original.action().author() == action.author();
            if own && chat::is_type(original.action(), UnitEntryTypes::Message)? {
                Ok(ValidateCallbackResult::Valid)
            } else {
                invalid("only a message's author can delete it, and nothing else is deleted")
            }
        }
        FlatOp::Link(OpLink::CreateLink { link_type, action })
        | FlatOp::CreateRecord(OpRecord::CreateLink { link_type, action }) => {
            chat::validate_link(link_type, &action)
        }
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

pub(crate) fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(reason.to_string()))
}
