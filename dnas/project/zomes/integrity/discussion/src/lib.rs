//! Talk inside one project. M4a: the project chat (ADR 0038). M4b: discussion threads, their
//! comments and up/down votes (ADR 0041). As on web, anyone signed in may write (the membrane
//! is open, ADR 0006), so nothing here reads membership. Nothing is ever edited (web has no
//! edit for chat, threads or comments); only an entry's author deletes it.

mod rules;

use hdi::prelude::*;
pub use rules::{decode_comment, decode_thread};

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ChatMessage {
    pub body: String,
    /// The message this one answers (web's `parent_id`), shown as a quote.
    pub reply_to: Option<ActionHash>,
}

/// A discussion thread: web's `threads` row, scoped to this project instead of channel tags.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Thread {
    pub title: String,
    /// The opening post.
    pub body: String,
}

/// A comment in a thread (web's `comments` with `subject_type = thread`).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Comment {
    pub thread: ActionHash,
    /// The comment this one replies to, in the same thread (web's `parent_id`).
    pub parent: Option<ActionHash>,
    pub body: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    ChatMessage(ChatMessage),
    Thread(Thread),
    Comment(Comment),
}

#[derive(Serialize, Deserialize)]
#[hdk_link_types]
pub enum LinkTypes {
    /// A chat anchor → a message, linked by the message's author. The coordinator chooses the
    /// anchor (one "chat" path today), so bucketing it by time later is a coordinator change.
    ChatMessages,
    /// The "threads" anchor → a thread, linked by the thread's author.
    Threads,
    /// A thread → one of its comments, linked by the comment's author.
    ThreadComments,
    /// A thread or comment → the voter's own key; the tag is up or down (`sp_core::discussion`).
    /// One live vote per agent and target is a coordinator rule; reads keep each voter's latest.
    Votes,
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
            EntryTypes::ChatMessage(m) => rules::validate_message(&m),
            EntryTypes::Thread(t) => rules::validate_thread(&t),
            EntryTypes::Comment(c) => rules::validate_comment(&c),
        },
        FlatOp::CreateEntry(OpEntry::UpdateEntry { .. })
        | FlatOp::CreateRecord(OpRecord::UpdateEntry { .. })
        | FlatOp::Update(_) => invalid("talk is never edited"),
        FlatOp::CreateRecord(OpRecord::DeleteEntry { action })
        | FlatOp::Delete(OpDelete { action }) => {
            let original = must_get_action(action.deletes_address.clone())?;
            if original.action().author() == action.author() {
                Ok(ValidateCallbackResult::Valid)
            } else {
                invalid("only its author can delete it")
            }
        }
        FlatOp::Link(OpLink::CreateLink { link_type, action })
        | FlatOp::CreateRecord(OpRecord::CreateLink { link_type, action }) => {
            rules::validate_link(link_type, &action)
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

/// The action creates an entry of `kind` in this zome (never decided by guessing from bytes).
pub fn is_entry(action: &Action, kind: UnitEntryTypes) -> ExternResult<bool> {
    let scoped: ScopedEntryDefIndex = kind.try_into()?;
    Ok(matches!(
        (action.action_type(), action.entry_type()),
        (ActionType::Create, Some(EntryType::App(def)))
            if def.zome_index == scoped.zome_index && def.entry_index == scoped.zome_type
    ))
}

/// The action creates a `ChatMessage`.
pub fn is_message(action: &Action) -> ExternResult<bool> {
    is_entry(action, UnitEntryTypes::ChatMessage)
}

/// The action creates a `Thread`.
pub fn is_thread(action: &Action) -> ExternResult<bool> {
    is_entry(action, UnitEntryTypes::Thread)
}

/// The action creates a `Comment`.
pub fn is_comment(action: &Action) -> ExternResult<bool> {
    is_entry(action, UnitEntryTypes::Comment)
}

fn invalid(reason: &str) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(reason.to_string()))
}
