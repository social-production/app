//! Pinned messages (web's `conversation_pins`, at most 5): either person pins in a direct
//! chat, the manager in a group. The newest pin or unpin of a message by someone allowed at
//! the time counts.

use std::collections::BTreeMap;

use hdk::prelude::*;
use messages_integrity::{EntryTypes, LinkTypes, Pin, UnitEntryTypes};
use sp_core::chat::preview;
use sp_core::conversation::ChatKind;

use crate::entries::{all, path};
use crate::people::{load, Chat};
use crate::{guest, reads, MESSAGES};

/// web's `MAX_PINS`.
const PINS_MAX: usize = 5;

#[derive(Serialize, Deserialize, Debug)]
pub struct PinInput {
    pub id: ActionHash,
    pub pinned: bool,
}

/// One pinned message, as web's pin bar shows it.
#[derive(Serialize, Deserialize, Debug)]
pub struct PinView {
    pub message: ActionHash,
    pub preview: String,
    pub pinned_at: Timestamp,
}

/// Pinned messages and when they were pinned.
pub fn pinned(chat: &Chat) -> ExternResult<BTreeMap<ActionHash, Timestamp>> {
    let mut state: BTreeMap<ActionHash, (bool, Timestamp)> = BTreeMap::new();
    for f in all::<Pin>(UnitEntryTypes::Pin, LinkTypes::Pins, MESSAGES)? {
        let allowed = match chat.kind {
            ChatKind::Direct => chat.roster.in_at(&f.author, f.at),
            ChatKind::Group => chat.roster.manager_at(f.at) == Some(&f.author),
        };
        if allowed {
            state.insert(f.entry.message, (f.entry.pinned, f.at));
        }
    }
    Ok(state
        .into_iter()
        .filter_map(|(id, (on, at))| on.then_some((id, at)))
        .collect())
}

/// web's `pin_message` / `unpin_message`.
#[hdk_extern]
pub fn pin_message(input: PinInput) -> ExternResult<()> {
    let chat = load()?;
    chat.require_member()?;
    let by = chat.pin_authority()?;
    let current = pinned(&chat)?;
    if current.contains_key(&input.id) == input.pinned {
        return Ok(());
    }
    if input.pinned && current.len() >= PINS_MAX {
        return Err(guest(&format!("You can pin at most {PINS_MAX} messages")));
    }
    let pin = Pin {
        message: input.id,
        pinned: input.pinned,
        by,
    };
    let hash = create_entry(EntryTypes::Pin(pin))?;
    create_link(path(MESSAGES)?, hash, LinkTypes::Pins, ())?;
    Ok(())
}

/// The pinned messages, oldest pin first (web's `list_pins`).
#[hdk_extern]
pub fn pins() -> ExternResult<Vec<PinView>> {
    let chat = load()?;
    let mut list: Vec<(ActionHash, Timestamp)> = pinned(&chat)?.into_iter().collect();
    list.sort_by_key(|(_, at)| *at);
    let views = reads::list(&chat)?;
    let mut out = Vec::new();
    for (message, pinned_at) in list {
        let shown = views
            .iter()
            .find(|v| v.id == message)
            .map(|v| preview(&v.body));
        // Older than the page: show it unless its author deleted it (web drops those pins).
        let preview = match shown {
            Some(p) => Some(p),
            None => match get_details(message.clone(), GetOptions::default())? {
                Some(Details::Record(d)) if d.deletes.is_empty() => {
                    reads::decode(&d.record)?.map(|m| preview(&m.body))
                }
                _ => None,
            },
        };
        if let Some(preview) = preview {
            out.push(PinView {
                message,
                preview,
                pinned_at,
            });
        }
    }
    Ok(out)
}
