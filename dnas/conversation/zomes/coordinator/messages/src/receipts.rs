//! Read receipts in a direct chat (web's `peer_read_at`: ✓ sent, ✓✓ read). Reading writes a
//! public `Seen` next to the private read mark; the other person's newest one is how far they
//! have read. Groups have none, as on web.

use hdk::prelude::*;
use messages_integrity::{is_type, EntryTypes, LinkTypes, Seen, UnitEntryTypes};
use sp_core::conversation::ChatKind;

use crate::entries::path;
use crate::people::{load, Chat};
use crate::reads::records;
use crate::MESSAGES;

/// Tell the other person in a direct chat that you have read up to `at`.
pub fn mark(chat: &Chat, at: Timestamp) -> ExternResult<()> {
    if chat.kind != ChatKind::Direct || !chat.is_member() {
        return Ok(());
    }
    let invite = chat.my_invite()?;
    let hash = create_entry(EntryTypes::Seen(Seen { at, invite }))?;
    create_link(path(MESSAGES)?, hash, LinkTypes::Seens, ())?;
    Ok(())
}

/// How far the other person in a direct chat has read (web's `peer_read_at`); `None` in a
/// group, or before they have read anything.
#[hdk_extern]
pub fn peer_read_at() -> ExternResult<Option<Timestamp>> {
    let chat = load()?;
    if chat.kind != ChatKind::Direct {
        return Ok(None);
    }
    let links = get_links(
        LinkQuery::try_new(path(MESSAGES)?, LinkTypes::Seens)?,
        GetStrategy::default(),
    )?;
    // Only their newest mark matters, so only its record is fetched.
    let Some(newest) = links
        .into_iter()
        .filter(|l| l.author != chat.me)
        .max_by_key(|l| l.timestamp)
        .and_then(|l| l.target.into_action_hash())
    else {
        return Ok(None);
    };
    let Some(record) = records(&[newest])?.into_iter().flatten().next() else {
        return Ok(None);
    };
    if !is_type(record.action(), UnitEntryTypes::Seen)? {
        return Ok(None);
    }
    Ok(record
        .entry()
        .to_app_option::<Seen>()
        .map_err(|e| wasm_error!(e))?
        .map(|s| s.at))
}
