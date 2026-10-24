//! Direct messages and group chats (ADR 0043, 0045, docs/plans/M4-talk.md "M4c"). In a chat
//! clone: `start_chat`, the group's `rename_group`/`add_member`/`remove_member`/`leave`, messages
//! (`post_message`, `edit_message`, `delete_message`, `list_messages`), pins (`pin_message`,
//! `pins`), and your own `summary`, `mark_read` and `set_prefs`; a direct chat's
//! `peer_read_at` (read receipts) and `log_call` (call lines). In the base cell (the inbox):
//! `send_invite` and `inbox`. The UI polls, as it does the project chat (ADR 0038).

mod calls;
mod entries;
mod files;
mod group;
mod inbox;
mod own;
mod people;
mod pins;
mod reads;
mod receipts;
mod summary;

use hdk::prelude::*;
use messages_integrity::{Attachment, Edit, EntryTypes, LinkTypes, Message};
use sp_attachment::{check_body, preview_text};
use sp_core::chat::{check_message, preview};

pub(crate) const MESSAGES: &str = "messages";

#[derive(Serialize, Deserialize, Debug)]
pub struct PostInput {
    pub body: String,
    pub reply_to: Option<ActionHash>,
    /// Photos and files whose chunks the caller has written (`upload_chunk`), ADR 0049.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// One message as the chat shows it (the project chat's shape).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub body: String,
    pub created_at: Timestamp,
    pub reply_to: Option<ReplyRef>,
    /// "text", "system" for a leave (web's "… left the group."; the UI names who), or "call"
    /// for a finished voice call (ADR 0051).
    pub kind: String,
    pub edited: bool,
    pub pinned: bool,
    pub attachments: Vec<Attachment>,
    /// A call row's outcome and length.
    pub call: Option<calls::CallLine>,
}

impl MessageView {
    /// What a list row or a pin shows: the text, else "Photo" or the first file's name.
    #[must_use]
    pub fn preview(&self) -> String {
        let first = self.attachments.first();
        preview(&preview_text(
            &self.body,
            first.map(|a| (a.kind.as_str(), a.name.as_str())),
        ))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditInput {
    pub id: ActionHash,
    pub body: String,
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
    check_body(&body, input.attachments.len()).map_err(|reason| guest(&reason))?;
    files::check(&input.attachments)?;
    let chat = people::load()?;
    chat.require_member()?;
    let invite = chat.my_invite()?;
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
        attachments: input.attachments,
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
    let chunks = files::chunks_of(&mine, &id)?;
    delete_entry(id)?;
    for chunk in chunks {
        delete_entry(chunk)?;
    }
    Ok(())
}

/// web's `edit_message`: a new text for your own message.
#[hdk_extern]
pub fn edit_message(input: EditInput) -> ExternResult<()> {
    let body = input.body.trim().to_string();
    check_message(&body).map_err(|_| guest("Message body is required"))?;
    people::load()?.require_member()?;
    if !own::messages()?
        .iter()
        .any(|r| r.action_address() == &input.id)
    {
        return Err(guest("You can only edit your own message."));
    }
    let hash = create_entry(EntryTypes::Edit(Edit {
        message: input.id,
        body,
    }))?;
    create_link(anchor()?, hash, LinkTypes::Edits, ())?;
    Ok(())
}

/// The newest messages (up to `sp_core::chat::PAGE_MAX`), oldest first.
#[hdk_extern]
pub fn list_messages() -> ExternResult<Vec<MessageView>> {
    reads::list(&people::load()?)
}
