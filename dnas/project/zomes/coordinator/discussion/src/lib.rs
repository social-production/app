//! Project talk API (docs/plans/M4-talk.md). The chat (ADR 0038): post, delete your own, and
//! read the newest page; the UI polls `list_messages` while the chat is open. Threads,
//! comments and votes (ADR 0041) live in `threads`, `comments` and `votes`. Reads merge the
//! network's view with the caller's own chain, so your own writes show at once.

mod comments;
mod own;
mod reads;
mod threads;
mod votes;

use discussion_integrity::{ChatMessage, EntryTypes, LinkTypes};
use hdk::prelude::*;
use sp_core::chat::check_message;

pub(crate) const CHAT: &str = "chat";

#[derive(Serialize, Deserialize, Debug)]
pub struct PostInput {
    pub body: String,
    pub reply_to: Option<ActionHash>,
}

/// One message as the chat shows it.
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
    Path::from(CHAT).path_entry_hash()
}

pub(crate) fn guest(reason: &str) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(reason.into()))
}

#[hdk_extern]
pub fn post_message(input: PostInput) -> ExternResult<ActionHash> {
    let body = input.body.trim().to_string();
    check_message(&body).map_err(|reason| guest(&reason))?;
    if let Some(parent) = &input.reply_to {
        if reads::record(parent)?.is_none() {
            return Err(guest(
                "The message you are replying to isn't here yet. Try again in a moment.",
            ));
        }
    }
    let message = ChatMessage {
        body,
        reply_to: input.reply_to,
    };
    let hash = create_entry(EntryTypes::ChatMessage(message))?;
    create_link(anchor()?, hash.clone(), LinkTypes::ChatMessages, ())?;
    Ok(hash)
}

/// Take back your own message: its chat link and the entry go (web's `delete_own_comment`).
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
