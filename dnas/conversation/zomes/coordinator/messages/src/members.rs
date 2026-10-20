//! Who is in a chat (the creator plus their invitations), its name, and how far you've read.

use std::collections::BTreeSet;

use hdk::prelude::*;
use messages_integrity::{
    is_type, network, EntryTypes, Invite, LinkTypes, Network, ReadMark, Title, UnitEntryTypes,
};
use sp_core::chat::preview;
use sp_core::conversation::{check_title, ChatKind, MEMBERS_MAX};

use crate::{guest, reads, MessageView};

#[derive(Serialize, Deserialize, Debug)]
pub struct StartInput {
    /// Everyone but the creator.
    pub members: Vec<AgentPubKey>,
    /// A group's name; ignored for a direct chat.
    pub title: Option<String>,
}

/// One row of web's chat list.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatSummary {
    /// "direct" or "group".
    pub kind: String,
    pub creator: AgentPubKey,
    /// Everyone in the chat, the creator first.
    pub members: Vec<AgentPubKey>,
    pub title: Option<String>,
    pub last: Option<LastMessage>,
    /// Messages from others since your last read mark.
    pub unread: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LastMessage {
    pub author: AgentPubKey,
    pub preview: String,
    pub created_at: Timestamp,
}

fn members_anchor() -> ExternResult<EntryHash> {
    Path::from("members").path_entry_hash()
}

fn chat() -> ExternResult<(AgentPubKey, ChatKind)> {
    match network()? {
        Network::Chat(creator, kind) => Ok((creator, kind)),
        Network::Inbox => Err(guest("This is the inbox, not a chat.")),
    }
}

/// Every invitation in the chat: the creator's own chain first, then the network.
fn invitations() -> ExternResult<Vec<(ActionHash, Invite)>> {
    let mut hashes: Vec<ActionHash> = query(
        ChainQueryFilter::new()
            .entry_type(EntryType::App(UnitEntryTypes::Invite.try_into()?))
            .action_type(ActionType::Create),
    )?
    .iter()
    .map(|r| r.action_address().clone())
    .collect();
    let links = get_links(
        LinkQuery::try_new(members_anchor()?, LinkTypes::Invites)?,
        GetStrategy::default(),
    )?;
    hashes.extend(
        links
            .into_iter()
            .filter_map(|l| l.target.into_action_hash()),
    );
    let unique: BTreeSet<ActionHash> = hashes.into_iter().collect();
    let wanted: Vec<ActionHash> = unique.into_iter().collect();
    let mut out = Vec::new();
    for record in reads::records(&wanted)?.into_iter().flatten() {
        if !is_type(record.action(), UnitEntryTypes::Invite)? {
            continue;
        }
        if let Some(invite) = record
            .entry()
            .to_app_option::<Invite>()
            .map_err(|e| wasm_error!(e))?
        {
            out.push((record.action_address().clone(), invite));
        }
    }
    Ok(out)
}

/// The caller's invitation, which their messages cite; `None` for the creator.
pub fn my_invite() -> ExternResult<Option<ActionHash>> {
    let (creator, _) = chat()?;
    let me = agent_info()?.agent_initial_pubkey;
    if me == creator {
        return Ok(None);
    }
    invitations()?
        .into_iter()
        .find(|(_, i)| i.agent == me)
        .map(|(hash, _)| Some(hash))
        .ok_or_else(|| {
            guest("Your invitation to this chat hasn't arrived yet. Try again in a moment.")
        })
}

/// The creator invites everyone else and, for a group, names it. Running it again invites
/// only the people not invited yet.
#[hdk_extern]
pub fn start_chat(input: StartInput) -> ExternResult<()> {
    let (creator, kind) = chat()?;
    let me = agent_info()?.agent_initial_pubkey;
    if me != creator {
        return Err(guest(
            "Only the person who started this chat can invite people.",
        ));
    }
    let invited: BTreeSet<AgentPubKey> = invitations()?.into_iter().map(|(_, i)| i.agent).collect();
    let mut wanted: Vec<AgentPubKey> = Vec::new();
    for agent in input.members {
        if agent != me && !invited.contains(&agent) && !wanted.contains(&agent) {
            wanted.push(agent);
        }
    }
    let total = 1 + invited.len() + wanted.len();
    if kind == ChatKind::Direct && total > 2 {
        return Err(guest("A direct chat is between two people."));
    }
    if total > MEMBERS_MAX {
        return Err(guest(&format!("A chat has at most {MEMBERS_MAX} people.")));
    }
    if kind == ChatKind::Group {
        if let Some(title) = input.title.map(|t| t.trim().to_string()) {
            check_title(&title).map_err(|reason| guest(&reason))?;
            let hash = create_entry(EntryTypes::Title(Title { title }))?;
            create_link(members_anchor()?, hash, LinkTypes::Titles, ())?;
        }
    }
    for agent in wanted {
        let hash = create_entry(EntryTypes::Invite(Invite { agent }))?;
        create_link(members_anchor()?, hash, LinkTypes::Invites, ())?;
    }
    Ok(())
}

/// The group's newest name.
fn title() -> ExternResult<Option<String>> {
    let links = get_links(
        LinkQuery::try_new(members_anchor()?, LinkTypes::Titles)?,
        GetStrategy::default(),
    )?;
    let mut hashes: Vec<ActionHash> = query(
        ChainQueryFilter::new()
            .entry_type(EntryType::App(UnitEntryTypes::Title.try_into()?))
            .action_type(ActionType::Create),
    )?
    .iter()
    .map(|r| r.action_address().clone())
    .collect();
    hashes.extend(
        links
            .into_iter()
            .filter_map(|l| l.target.into_action_hash()),
    );
    let mut newest: Option<(Timestamp, String)> = None;
    for record in reads::records(&hashes)?.into_iter().flatten() {
        let Some(t) = record
            .entry()
            .to_app_option::<Title>()
            .map_err(|e| wasm_error!(e))?
        else {
            continue;
        };
        let at = record.action().timestamp();
        if newest.as_ref().is_none_or(|(n, _)| at > *n) {
            newest = Some((at, t.title));
        }
    }
    Ok(newest.map(|(_, t)| t))
}

/// When the caller last marked this chat read.
fn read_at() -> ExternResult<Option<Timestamp>> {
    let marks = query(
        ChainQueryFilter::new()
            .entry_type(EntryType::App(UnitEntryTypes::ReadMark.try_into()?))
            .include_entries(true),
    )?;
    let mut newest = None;
    for r in marks {
        if let Some(m) = r
            .entry()
            .to_app_option::<ReadMark>()
            .map_err(|e| wasm_error!(e))?
        {
            newest = newest.max(Some(m.at));
        }
    }
    Ok(newest)
}

fn unread(messages: &[MessageView], me: &AgentPubKey, since: Option<Timestamp>) -> usize {
    messages
        .iter()
        .filter(|m| &m.author != me && since.is_none_or(|at| m.created_at > at))
        .count()
}

#[hdk_extern]
pub fn summary() -> ExternResult<ChatSummary> {
    let (creator, kind) = chat()?;
    let me = agent_info()?.agent_initial_pubkey;
    let mut members = vec![creator.clone()];
    for (_, invite) in invitations()? {
        if !members.contains(&invite.agent) {
            members.push(invite.agent);
        }
    }
    let messages = reads::list()?;
    let last = messages.last().map(|m| LastMessage {
        author: m.author.clone(),
        preview: preview(&m.body),
        created_at: m.created_at,
    });
    Ok(ChatSummary {
        kind: kind.as_str().into(),
        creator,
        members,
        title: if kind == ChatKind::Group {
            title()?
        } else {
            None
        },
        last,
        unread: u32::try_from(unread(&messages, &me, read_at()?)).unwrap_or(u32::MAX),
    })
}

/// Mark everything shown as read (web's `mark_conversation_as_read`). Writes a private mark
/// only when something from someone else is unread.
#[hdk_extern]
pub fn mark_read() -> ExternResult<()> {
    chat()?;
    let me = agent_info()?.agent_initial_pubkey;
    let messages = reads::list()?;
    if unread(&messages, &me, read_at()?) > 0 {
        // A sender's clock may run ahead of ours: the mark covers the newest message.
        let newest = messages.iter().map(|m| m.created_at).max();
        let at = newest.max(Some(sys_time()?)).unwrap_or(sys_time()?);
        create_entry(EntryTypes::ReadMark(ReadMark { at }))?;
    }
    Ok(())
}
