//! Who may write what, in a chat and in the inbox (ADR 0043).

use hdi::prelude::*;
use sp_core::chat::check_message;
use sp_core::conversation::{check_title, ChatKind, INBOX_TAG_MAX};

use crate::{invalid, network, EntryTypes, Invite, LinkTypes, Message, Network, UnitEntryTypes};

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
    match entry {
        EntryTypes::Invite(_) if author != &creator => invalid("only the chat's creator invites"),
        EntryTypes::Invite(i) if i.agent == creator => invalid("the creator is already in"),
        EntryTypes::Invite(_) => Ok(ValidateCallbackResult::Valid),
        EntryTypes::Message(m) => validate_message(m, author, &creator),
        EntryTypes::Title(_) if kind != ChatKind::Group => invalid("only groups have a name"),
        EntryTypes::Title(_) if author != &creator => invalid("only the creator names a group"),
        EntryTypes::Title(t) => match check_title(&t.title) {
            Ok(()) => Ok(ValidateCallbackResult::Valid),
            Err(reason) => invalid(&reason),
        },
        EntryTypes::ReadMark(_) => validate_private(author),
    }
}

/// Read marks stay on their author's chain; they only have to be in a chat.
pub fn validate_private(_author: &AgentPubKey) -> ExternResult<ValidateCallbackResult> {
    match chat()? {
        Ok(_) => Ok(ValidateCallbackResult::Valid),
        Err(reason) => invalid(&reason),
    }
}

fn validate_message(
    m: &Message,
    author: &AgentPubKey,
    creator: &AgentPubKey,
) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check_message(&m.body) {
        return invalid(&reason);
    }
    if author != creator {
        let Some(invite) = &m.invite else {
            return invalid("only people invited to the chat write in it");
        };
        if !invites(invite, author, creator)? {
            return invalid("that invitation isn't for the message's author");
        }
    }
    if let Some(parent) = &m.reply_to {
        let record = must_get_valid_record(parent.clone())?;
        if !is_type(record.action(), UnitEntryTypes::Message)? {
            return invalid("a reply answers a message");
        }
    }
    Ok(ValidateCallbackResult::Valid)
}

/// `invite` is the creator's invitation of `agent`.
fn invites(invite: &ActionHash, agent: &AgentPubKey, creator: &AgentPubKey) -> ExternResult<bool> {
    let record = must_get_valid_record(invite.clone())?;
    if record.action().author() != creator || !is_type(record.action(), UnitEntryTypes::Invite)? {
        return Ok(false);
    }
    let invite: Option<Invite> = record.entry().to_app_option().map_err(|e| wasm_error!(e))?;
    Ok(invite.is_some_and(|i| &i.agent == agent))
}

pub fn validate_link(
    link_type: LinkTypes,
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    if matches!(link_type, LinkTypes::Inbox) {
        return validate_inbox(action);
    }
    let (creator, _) = match chat()? {
        Ok(chat) => chat,
        Err(reason) => return invalid(&reason),
    };
    let Some(target) = action.target_address.clone().into_action_hash() else {
        return invalid("a chat link points at an entry's action");
    };
    let signed = must_get_action(target)?;
    let (want, by_creator) = match link_type {
        LinkTypes::Invites => (UnitEntryTypes::Invite, true),
        LinkTypes::Titles => (UnitEntryTypes::Title, true),
        LinkTypes::Messages => (UnitEntryTypes::Message, false),
        LinkTypes::Inbox => unreachable!("handled above"),
    };
    let linker_ok = if by_creator {
        action.author() == &creator
    } else {
        true
    };
    if is_type(signed.action(), want)? && signed.action().author() == action.author() && linker_ok {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid(
            "people list only their own entries, and only the creator lists invitations and names",
        )
    }
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
