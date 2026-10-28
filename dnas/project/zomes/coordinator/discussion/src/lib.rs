//! Project chat API (docs/plans/M4-talk.md, ADR 0038): post, delete your own, and read the
//! newest page. Reads merge the network's view with the caller's own chain, so your own
//! message (or deletion) shows at once. The UI polls `list_messages` while the chat is open.
//! Photos and files and a private read mark came with ADR 0056.

mod files;
mod own;
mod reads;
mod scope;

use discussion_integrity::{Attachment, ChatMessage, EntryTypes, LinkTypes, ReadMark};
use hdk::prelude::*;
use sp_attachment::check_body;

pub use files::Bytes;
use scope::{enter, In};

#[derive(Serialize, Deserialize, Debug)]
pub struct PostInput {
    pub body: String,
    pub reply_to: Option<ActionHash>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// One message as the chat shows it.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub body: String,
    pub created_at: Timestamp,
    pub reply_to: Option<ReplyRef>,
    pub attachments: Vec<Attachment>,
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

/// A space's chat hangs off the space itself (its charter hash, ADR 0065).
pub(crate) fn anchor() -> ExternResult<ActionHash> {
    scope::current()
}

fn guest(reason: &str) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(reason.into()))
}

#[hdk_extern]
pub fn post_message(call: In<PostInput>) -> ExternResult<ActionHash> {
    let input = enter(call)?;
    let body = input.body.trim().to_string();
    check_body(&body, input.attachments.len()).map_err(|reason| guest(&reason))?;
    files::check(&input.attachments)?;
    if let Some(parent) = &input.reply_to {
        if reads::record(parent)?.is_none() {
            return Err(guest(
                "The message you are replying to isn't here yet. Try again in a moment.",
            ));
        }
    }
    let message = ChatMessage {
        space: anchor()?,
        body,
        reply_to: input.reply_to,
        attachments: input.attachments,
    };
    let hash = create_entry(EntryTypes::ChatMessage(message))?;
    create_link(anchor()?, hash.clone(), LinkTypes::ChatMessages, ())?;
    scope::touch()?;
    Ok(hash)
}

/// Take back your own message: its chat link and the entry go (web's `delete_own_comment`).
#[hdk_extern]
pub fn delete_message(call: In<ActionHash>) -> ExternResult<()> {
    let id = enter(call)?;
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
    for chunk in files::chunks_of(&mine, &id)? {
        delete_entry(chunk)?;
    }
    delete_entry(id)?;
    Ok(())
}

/// One part of a file you're about to send (at most 1 MB); its hash goes in the message.
#[hdk_extern]
pub fn upload_chunk(bytes: Bytes) -> ExternResult<ActionHash> {
    files::upload(bytes)
}

/// A part of a file in this chat; `None` while it can't be reached.
#[hdk_extern]
pub fn get_chunk(hash: ActionHash) -> ExternResult<Option<Bytes>> {
    files::chunk(hash)
}

/// Mark the chat read up to `at` (private to you, on every device you sign in on).
#[hdk_extern]
pub fn mark_read(call: In<Timestamp>) -> ExternResult<()> {
    let at = enter(call)?;
    if own::read_at()?.is_none_or(|old| at > old) {
        let space = anchor()?;
        create_entry(EntryTypes::ReadMark(ReadMark { space, at }))?;
    }
    Ok(())
}

/// How far you've read this chat, from your own chain.
#[hdk_extern]
pub fn read_at(call: In<()>) -> ExternResult<Option<Timestamp>> {
    enter(call)?;
    own::read_at()
}

/// Every space whose chat the caller wrote in (web's Public tab lists those too).
#[hdk_extern]
pub fn my_chat_spaces(_: ()) -> ExternResult<Vec<ActionHash>> {
    own::spaces()
}

/// The newest messages (up to `sp_core::chat::PAGE_MAX`), oldest first.
#[hdk_extern]
pub fn list_messages(call: In<()>) -> ExternResult<Vec<MessageView>> {
    enter(call)?;
    reads::list()
}
