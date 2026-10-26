//! Project chat API (docs/plans/M4-talk.md, ADR 0038): post, delete your own, and read the
//! newest page. Reads merge the network's view with the caller's own chain, so your own
//! message (or deletion) shows at once. The UI polls `list_messages` while the chat is open.
//! Photos and files and a private read mark came with ADR 0056.

mod files;
mod own;
mod reads;

use discussion_integrity::{
    Attachment, ChatMessage, EntryTypes, LinkTypes, ReadMark, UnitEntryTypes,
};
use hdk::prelude::*;
use sp_attachment::check_body;

pub use files::Bytes;

pub(crate) const CHAT: &str = "chat";

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

pub(crate) fn anchor() -> ExternResult<EntryHash> {
    Path::from(CHAT).path_entry_hash()
}

fn guest(reason: &str) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(reason.into()))
}

#[hdk_extern]
pub fn post_message(input: PostInput) -> ExternResult<ActionHash> {
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
        body,
        reply_to: input.reply_to,
        attachments: input.attachments,
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
pub fn mark_read(at: Timestamp) -> ExternResult<()> {
    if read_at(())?.is_none_or(|old| at > old) {
        create_entry(EntryTypes::ReadMark(ReadMark { at }))?;
    }
    Ok(())
}

/// How far you've read this chat, from your own chain.
#[hdk_extern]
pub fn read_at(_: ()) -> ExternResult<Option<Timestamp>> {
    let marks = query(
        ChainQueryFilter::new()
            .entry_type(EntryType::App(UnitEntryTypes::ReadMark.try_into()?))
            .include_entries(true),
    )?;
    Ok(marks
        .iter()
        .filter_map(|r| r.entry().to_app_option::<ReadMark>().ok().flatten())
        .map(|m| m.at)
        .max())
}

/// The newest messages (up to `sp_core::chat::PAGE_MAX`), oldest first.
#[hdk_extern]
pub fn list_messages() -> ExternResult<Vec<MessageView>> {
    reads::list()
}
