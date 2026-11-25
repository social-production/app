//! One row of web's chat list, how far you've read, and your own list settings (pin, mute,
//! hide), all private to you except the chat's name and members.

use hdk::prelude::*;
use messages_integrity::{EntryTypes, LinkTypes, Picture, Prefs, ReadMark, Title, UnitEntryTypes};
use sp_core::conversation::ChatKind;

use crate::entries::{all, private};
use crate::files::Bytes;
use crate::people::{load, Chat, MEMBERS};
use crate::{reads, receipts, MessageView};

/// A view for the UI, so its flags are plain fields (web's row has the same ones).
#[allow(clippy::struct_excessive_bools)]
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatSummary {
    /// "direct" or "group".
    pub kind: String,
    pub creator: AgentPubKey,
    /// Who manages a group now (web's `created_by` after handovers): the longest-managing.
    pub manager: Option<AgentPubKey>,
    /// Everyone who manages it now (ADR 0122).
    pub managers: Vec<AgentPubKey>,
    /// The group's photo (a small JPEG), when a manager set one.
    pub picture: Option<Bytes>,
    /// Everyone in the chat now, longest-standing first.
    pub members: Vec<AgentPubKey>,
    pub title: Option<String>,
    pub last: Option<LastMessage>,
    /// Messages from others since your last read mark.
    pub unread: u32,
    /// You're in the chat (not removed, not left).
    pub member: bool,
    /// You may pin messages here (web's `can_pin`).
    pub can_pin: bool,
    pub pinned: bool,
    pub muted: bool,
    /// You deleted it from your list and nothing newer has arrived.
    pub hidden: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LastMessage {
    pub author: AgentPubKey,
    pub preview: String,
    pub created_at: Timestamp,
    /// "text", "system" for "… left the group.", or "call" (an empty preview, as on web).
    pub kind: String,
}

/// web's `set_conversation_preferences`: only the fields given change.
#[derive(Serialize, Deserialize, Debug)]
pub struct PrefsInput {
    pub pinned: Option<bool>,
    pub muted: Option<bool>,
    pub hidden: Option<bool>,
}

/// The group's newest name by whoever managed it at the time.
fn title(chat: &Chat) -> ExternResult<Option<String>> {
    Ok(
        all::<Title>(UnitEntryTypes::Title, LinkTypes::Titles, MEMBERS)?
            .into_iter()
            .rfind(|f| chat.roster.managed_at(&f.author, f.at))
            .map(|f| f.entry.title),
    )
}

/// The group's newest photo by whoever managed it at the time (ADR 0122).
fn picture(chat: &Chat) -> ExternResult<Option<Bytes>> {
    Ok(
        all::<Picture>(UnitEntryTypes::Picture, LinkTypes::Pictures, MEMBERS)?
            .into_iter()
            .rfind(|f| chat.roster.managed_at(&f.author, f.at))
            .map(|f| Bytes(f.entry.bytes)),
    )
}

fn read_at() -> ExternResult<Option<Timestamp>> {
    Ok(private::<ReadMark>(UnitEntryTypes::ReadMark)?
        .into_iter()
        .map(|f| f.entry.at)
        .max())
}

fn prefs() -> ExternResult<Prefs> {
    Ok(private::<Prefs>(UnitEntryTypes::Prefs)?.pop().map_or(
        Prefs {
            pinned: false,
            muted: false,
            hidden_at: None,
        },
        |f| f.entry,
    ))
}

fn unread(messages: &[MessageView], me: &AgentPubKey, since: Option<Timestamp>) -> usize {
    messages
        .iter()
        .filter(|m| &m.author != me && since.is_none_or(|at| m.created_at > at))
        .count()
}

#[hdk_extern]
pub fn summary() -> ExternResult<ChatSummary> {
    let chat = load()?;
    let messages = reads::list(&chat)?;
    let last = messages.last().map(|m| LastMessage {
        author: m.author.clone(),
        preview: m.preview(),
        created_at: m.created_at,
        kind: m.kind.clone(),
    });
    let prefs = prefs()?;
    let newest = messages.last().map(|m| m.created_at);
    let can_pin = chat.is_member() && (chat.kind == ChatKind::Direct || chat.is_manager());
    Ok(ChatSummary {
        kind: chat.kind.as_str().into(),
        creator: chat.creator.clone(),
        manager: chat.roster.manager().cloned(),
        managers: chat.roster.managers.clone(),
        picture: if chat.kind == ChatKind::Group {
            picture(&chat)?
        } else {
            None
        },
        members: chat.roster.members.clone(),
        title: if chat.kind == ChatKind::Group {
            title(&chat)?
        } else {
            None
        },
        last,
        unread: u32::try_from(unread(&messages, &chat.me, read_at()?)).unwrap_or(u32::MAX),
        member: chat.is_member(),
        can_pin,
        pinned: prefs.pinned,
        muted: prefs.muted,
        hidden: prefs
            .hidden_at
            .is_some_and(|at| newest.is_none_or(|n| n <= at)),
    })
}

/// Mark everything shown as read (web's `mark_conversation_as_read`). Writes a private mark
/// (and, in a direct chat, a read receipt) only when something from someone else is unread.
#[hdk_extern]
pub fn mark_read() -> ExternResult<()> {
    let chat = load()?;
    let messages = reads::list(&chat)?;
    if unread(&messages, &chat.me, read_at()?) > 0 {
        // A sender's clock may run ahead of ours: the mark covers the newest message.
        let newest = messages.iter().map(|m| m.created_at).max();
        let at = newest.max(Some(sys_time()?)).unwrap_or(sys_time()?);
        create_entry(EntryTypes::ReadMark(ReadMark { at }))?;
        receipts::mark(&chat, at)?;
    }
    Ok(())
}

#[hdk_extern]
pub fn set_prefs(input: PrefsInput) -> ExternResult<()> {
    let mut prefs = prefs()?;
    if let Some(pinned) = input.pinned {
        prefs.pinned = pinned;
    }
    if let Some(muted) = input.muted {
        prefs.muted = muted;
    }
    if let Some(hidden) = input.hidden {
        // A sender's clock may run ahead of ours: hide what is shown now, too.
        let newest = reads::list(&load()?)?.last().map(|m| m.created_at);
        prefs.hidden_at = hidden.then(|| newest.max(sys_time().ok())).flatten();
    }
    create_entry(EntryTypes::Prefs(prefs))?;
    Ok(())
}
