//! Direct messages and group chats (ADR 0043, docs/plans/M4-talk.md "M4c"). In a chat clone:
//! `start_chat` (the creator invites people and names a group), `post_message`, `delete_message`,
//! `list_messages`, `summary` and `mark_read`. In the base cell (the inbox): `send_invite` and
//! `inbox`. The UI polls, as it does the project chat (ADR 0038).

mod inbox;
mod members;
mod own;
mod reads;

use hdk::prelude::*;
use messages_integrity::{EntryTypes, LinkTypes, Message};
use sp_core::chat::check_message;

pub(crate) const MESSAGES: &str = "messages";

#[derive(Serialize, Deserialize, Debug)]
pub struct PostInput {
    pub body: String,
    pub reply_to: Option<ActionHash>,
}

/// One message as the chat shows it (the project chat's shape).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub body: String,
    pub created_at: Timestamp,
    pub reply_to: Option<ReplyRef>,
}

/// The quoted original of a reply.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyRef {
    pub id: ActionHash,
    /// `None` when the original can't be reached yet.
    pub author: Option<AgentPubKey>,
    pub preview: String,
    /// Its author deleted it: the quote reads "Deleted message", as on web.
    pub deleted: bool,
}

pub(crate) fn anchor() -> ExternResult<EntryHash> {
    Path::from(MESSAGES).path_entry_hash()
}

pub(crate) fn guest(reason: &str) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(reason.into()))
}

#[hdk_extern]
pub fn post_message(input: PostInput) -> ExternResult<ActionHash> {
    let body = input.body.trim().to_string();
    check_message(&body).map_err(|reason| guest(&reason))?;
    let invite = members::my_invite()?;
    if let Some(parent) = &input.reply_to {
        if reads::record(parent)?.is_none() {
            return Err(guest(
                "The message you are replying to isn't here yet. Try again in a moment.",
            ));
        }
    }
    let message = Message {
        body,
        reply_to: input.reply_to,
        invite,
    };
    let hash = create_entry(EntryTypes::Message(message))?;
    create_link(anchor()?, hash.clone(), LinkTypes::Messages, ())?;
    Ok(hash)
}

/// Take back your own message: its link and the entry go (web's `delete_message`).
#[hdk_extern]
pub fn delete_message(id: ActionHash) -> ExternResult<()> {
    let mine = own::messages()?;
    if !mine.iter().any(|r| r.action_address() == &id) {
        return Err(guest("You can only delete your own message."));
    }
    if own::deleted()?.contains(&id) {
        return Ok(());
    }
    for link in own::live_links_to(&id)? {
        delete_link(link, GetOptions::default())?;
    }
    delete_entry(id)?;
    Ok(())
}

/// The newest messages (up to `sp_core::chat::PAGE_MAX`), oldest first.
#[hdk_extern]
pub fn list_messages() -> ExternResult<Vec<MessageView>> {
    reads::list()
}
