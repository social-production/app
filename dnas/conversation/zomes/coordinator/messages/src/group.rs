//! Starting a chat and managing a group (web-backend's `conversations.py`): the manager
//! invites, renames and removes; anyone leaves, and a leaving manager hands over to the
//! longest-standing member (ADR 0045).

use hdk::prelude::*;
use messages_integrity::{EntryTypes, Invite, Leave, LinkTypes, Removal, Title};
use sp_core::conversation::{check_title, ChatKind, MEMBERS_MAX};

use crate::entries::path;
use crate::guest;
use crate::people::{load, Chat, MEMBERS};

#[derive(Serialize, Deserialize, Debug)]
pub struct StartInput {
    /// Everyone but the creator.
    pub members: Vec<AgentPubKey>,
    /// A group's name; ignored for a direct chat.
    pub title: Option<String>,
}

fn list(entry: EntryTypes, link: LinkTypes) -> ExternResult<ActionHash> {
    let hash = create_entry(entry)?;
    create_link(path(MEMBERS)?, hash.clone(), link, ())?;
    Ok(hash)
}

fn name(title: String, by: Option<ActionHash>) -> ExternResult<()> {
    let title = title.trim().to_string();
    check_title(&title).map_err(|_| guest("title is required"))?;
    list(EntryTypes::Title(Title { title, by }), LinkTypes::Titles)?;
    Ok(())
}

fn invite(chat: &Chat, agents: Vec<AgentPubKey>, by: Option<ActionHash>) -> ExternResult<()> {
    let mut wanted: Vec<AgentPubKey> = Vec::new();
    for agent in agents {
        if agent != chat.me && !chat.roster.members.contains(&agent) && !wanted.contains(&agent) {
            wanted.push(agent);
        }
    }
    let total = chat.roster.members.len() + wanted.len();
    if chat.kind == ChatKind::Direct && total > 2 {
        return Err(guest("A direct chat is between two people."));
    }
    if total > MEMBERS_MAX {
        return Err(guest(&format!("A chat has at most {MEMBERS_MAX} people.")));
    }
    for agent in wanted {
        let entry = EntryTypes::Invite(Invite {
            agent,
            by: by.clone(),
        });
        list(entry, LinkTypes::Invites)?;
    }
    Ok(())
}

/// The creator invites everyone else and, for a group, names it. Running it again invites
/// only the people not in yet.
#[hdk_extern]
pub fn start_chat(input: StartInput) -> ExternResult<()> {
    let chat = load()?;
    if chat.me != chat.creator {
        return Err(guest(
            "Only the person who started this chat can invite people.",
        ));
    }
    if chat.kind == ChatKind::Group {
        if let Some(title) = input.title {
            name(title, None)?;
        }
    }
    invite(&chat, input.members, None)
}

/// web's `rename_group_conversation`.
#[hdk_extern]
pub fn rename_group(title: String) -> ExternResult<()> {
    let chat = load()?;
    chat.require_group()?;
    name(title, chat.my_authority()?)
}

/// web's `add_group_member`. The caller then sends them the chat through the inbox.
#[hdk_extern]
pub fn add_member(agent: AgentPubKey) -> ExternResult<()> {
    let chat = load()?;
    chat.require_group()?;
    let by = chat.my_authority()?;
    invite(&chat, vec![agent], by)
}

/// web's `remove_group_member`.
#[hdk_extern]
pub fn remove_member(agent: AgentPubKey) -> ExternResult<()> {
    let chat = load()?;
    chat.require_group()?;
    let by = chat.my_authority()?;
    if agent == chat.me {
        return Err(guest("Creator cannot remove self"));
    }
    if !chat.roster.members.contains(&agent) {
        return Ok(());
    }
    list(
        EntryTypes::Removal(Removal { agent, by }),
        LinkTypes::Removals,
    )?;
    Ok(())
}

/// web's `leave_group`: a leaving manager hands the group to the longest-standing member.
#[hdk_extern]
pub fn leave() -> ExternResult<()> {
    let chat = load()?;
    if chat.kind != ChatKind::Group {
        return Err(guest(
            "Only group chats can be left. Delete a direct chat instead.",
        ));
    }
    if !chat.is_member() {
        return Ok(());
    }
    let (successor, by) = if chat.is_manager() {
        let next = chat.roster.members.iter().find(|m| **m != chat.me).cloned();
        (
            next.clone(),
            if next.is_some() {
                chat.my_authority()?
            } else {
                None
            },
        )
    } else {
        (None, None)
    };
    let entry = Leave {
        invite: chat.my_invite()?,
        successor,
        by,
    };
    list(EntryTypes::Leave(entry), LinkTypes::Leaves)?;
    Ok(())
}
