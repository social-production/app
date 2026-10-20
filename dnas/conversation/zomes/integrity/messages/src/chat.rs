//! Who may write what, in a chat and in the inbox (ADR 0043).

use hdi::prelude::*;
use sp_core::chat::check_message;
use sp_core::conversation::{check_title, ChatKind, INBOX_TAG_MAX};

use crate::rights::{manages, member, message_author};
use crate::{
    invalid, network, Edit, EntryTypes, Leave, LinkTypes, Message, Network, UnitEntryTypes,
};

/// The chat's creator and kind, or why this write doesn't belong in the inbox.
fn chat() -> ExternResult<Result<(AgentPubKey, ChatKind), String>> {
    Ok(match network()? {
        Network::Chat(creator, kind) => Ok((creator, kind)),
        Network::Inbox => Err("the inbox holds only invitations".into()),
    })
}

pub fn validate_entry(
    entry: &EntryTypes,
    author: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    let (creator, kind) = match chat()? {
        Ok(chat) => chat,
        Err(reason) => return invalid(&reason),
    };
    let group = kind == ChatKind::Group;
    let manager = |by: Option<&ActionHash>| manages(author, by, &creator);
    match entry {
        EntryTypes::Invite(i) if &i.agent == author => invalid("people don't invite themselves"),
        EntryTypes::Invite(i) => allow(manager(i.by.as_ref())?, "only the chat's manager invites"),
        EntryTypes::Message(m) => validate_message(m, author, &creator),
        EntryTypes::Title(_) if !group => invalid("only groups have a name"),
        EntryTypes::Title(t) => match check_title(&t.title) {
            Ok(()) => allow(manager(t.by.as_ref())?, "only the manager names a group"),
            Err(reason) => invalid(&reason),
        },
        EntryTypes::Removal(_) if !group => invalid("only groups remove people"),
        EntryTypes::Removal(r) if &r.agent == author => {
            invalid("leave instead of removing yourself")
        }
        EntryTypes::Removal(r) => allow(manager(r.by.as_ref())?, "only the manager removes people"),
        EntryTypes::Leave(_) if !group => invalid("only groups can be left"),
        EntryTypes::Leave(l) => validate_leave(l, author, &creator),
        EntryTypes::Edit(e) => validate_edit(e, author),
        EntryTypes::Pin(p) => {
            if message_author(&p.message)?.is_none() {
                return invalid("only messages are pinned");
            }
            let allowed = if group {
                manager(p.by.as_ref())?
            } else {
                member(author, p.by.as_ref(), &creator)?
            };
            allow(allowed, "in a group only the manager pins")
        }
        EntryTypes::ReadMark(_) | EntryTypes::Prefs(_) => validate_private(),
    }
}

fn allow(ok: bool, reason: &str) -> ExternResult<ValidateCallbackResult> {
    if ok {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid(reason)
    }
}

/// Read marks and list settings stay on their author's chain; they only have to be in a chat.
pub fn validate_private() -> ExternResult<ValidateCallbackResult> {
    match chat()? {
        Ok(_) => Ok(ValidateCallbackResult::Valid),
        Err(reason) => invalid(&reason),
    }
}

fn validate_leave(
    l: &Leave,
    author: &AgentPubKey,
    creator: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if !member(author, l.invite.as_ref(), creator)? {
        return invalid("only people in the group leave it");
    }
    match &l.successor {
        None => Ok(ValidateCallbackResult::Valid),
        Some(next) if next == author => invalid("a leaving manager hands over to someone else"),
        Some(_) => allow(
            manages(author, l.by.as_ref(), creator)?,
            "only the manager hands the group over",
        ),
    }
}

fn validate_edit(e: &Edit, author: &AgentPubKey) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_message(&e.body) {
        return invalid(&reason);
    }
    allow(
        message_author(&e.message)?.as_ref() == Some(author),
        "people edit only their own messages",
    )
}

fn validate_message(
    m: &Message,
    author: &AgentPubKey,
    creator: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_message(&m.body) {
        return invalid(&reason);
    }
    if !member(author, m.invite.as_ref(), creator)? {
        return invalid("only people invited to the chat write in it");
    }
    if let Some(parent) = &m.reply_to {
        if message_author(parent)?.is_none() {
            return invalid("a reply answers a message");
        }
    }
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_link(
    link_type: LinkTypes,
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    if matches!(link_type, LinkTypes::Inbox) {
        return validate_inbox(action);
    }
    if let Err(reason) = chat()? {
        return invalid(&reason);
    }
    let Some(target) = action.target_address.clone().into_action_hash() else {
        return invalid("a chat link points at an entry's action");
    };
    let signed = must_get_action(target)?;
    let want = match link_type {
        LinkTypes::Invites => UnitEntryTypes::Invite,
        LinkTypes::Titles => UnitEntryTypes::Title,
        LinkTypes::Messages => UnitEntryTypes::Message,
        LinkTypes::Removals => UnitEntryTypes::Removal,
        LinkTypes::Leaves => UnitEntryTypes::Leave,
        LinkTypes::Edits => UnitEntryTypes::Edit,
        LinkTypes::Pins => UnitEntryTypes::Pin,
        LinkTypes::Inbox => unreachable!("handled above"),
    };
    // Whether the entry itself was allowed is the entry's own validation.
    allow(
        is_type(signed.action(), want)? && signed.action().author() == action.author(),
        "people list only their own entries, each under its own link",
    )
}

/// An invitation sits on the invited person's key, comes from the sender's key, and is small.
fn validate_inbox(action: &TypedAction<CreateLinkData>) -> ExternResult<ValidateCallbackResult> {
    if let Network::Chat(..) = network()? {
        return invalid("invitations go to the inbox");
    }
    if action.base_address.clone().into_agent_pub_key().is_none() {
        return invalid("an invitation sits on a person's key");
    }
    let from = action.target_address.clone().into_agent_pub_key();
    if from.as_ref() != Some(action.author()) {
        return invalid("an invitation points at its sender");
    }
    if action.tag.0.len() > INBOX_TAG_MAX {
        return invalid("an invitation is too large");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// The action creates an entry of this zome's `unit` type (never guessed from bytes).
pub fn is_type(action: &Action, unit: UnitEntryTypes) -> ExternResult<bool> {
    let scoped: ScopedEntryDefIndex = unit.try_into()?;
    Ok(matches!(
        (action.action_type(), action.entry_type()),
        (ActionType::Create, Some(EntryType::App(def)))
            if def.zome_index == scoped.zome_index && def.entry_index == scoped.zome_type
    ))
}
