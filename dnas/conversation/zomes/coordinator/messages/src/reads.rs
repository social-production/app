//! Reading a chat: the newest linked messages plus the caller's own, with reply quotes.

use std::collections::{BTreeMap, BTreeSet};

use hdk::prelude::*;
use messages_integrity::{is_type, Edit, LinkTypes, Message, UnitEntryTypes};
use sp_core::chat::{preview, PAGE_MAX};

use crate::entries::all;
use crate::people::Chat;
use crate::{anchor, own, pins, MessageView, ReplyRef, MESSAGES};

/// One record: the local copy first (messages never change once written), then the network.
pub fn record(hash: &ActionHash) -> ExternResult<Option<Record>> {
    Ok(records(std::slice::from_ref(hash))?
        .into_iter()
        .flatten()
        .next())
}

pub fn batch(hashes: &[ActionHash], options: &GetOptions) -> ExternResult<Vec<Option<Record>>> {
    let inputs = hashes
        .iter()
        .map(|h| GetInput::new(h.clone().into(), options.clone()))
        .collect();
    HDK.with(|h| h.borrow().get(inputs))
}

/// Many records in two host calls at most: whatever is held here, then the rest.
pub fn records(hashes: &[ActionHash]) -> ExternResult<Vec<Option<Record>>> {
    let mut out = batch(hashes, &GetOptions::local())?;
    let missing: Vec<usize> = (0..out.len()).filter(|&i| out[i].is_none()).collect();
    if missing.is_empty() {
        return Ok(out);
    }
    let wanted: Vec<ActionHash> = missing.iter().map(|&i| hashes[i].clone()).collect();
    for (i, found) in missing
        .into_iter()
        .zip(batch(&wanted, &GetOptions::network())?)
    {
        out[i] = found;
    }
    Ok(out)
}

pub fn decode(record: &Record) -> ExternResult<Option<Message>> {
    if !is_type(record.action(), UnitEntryTypes::Message)? {
        return Ok(None);
    }
    record
        .entry()
        .to_app_option::<Message>()
        .map_err(|e| wasm_error!(e))
}

/// The newest `PAGE_MAX` messages, oldest first, with edits applied, pins marked and a
/// system row for each leave (web's "… left the group."). Deleted ones are gone with their
/// link; messages written while their author was out of the chat don't count (ADR 0045).
pub fn list(chat: &Chat) -> ExternResult<Vec<MessageView>> {
    let edits: BTreeMap<ActionHash, String> =
        all::<Edit>(UnitEntryTypes::Edit, LinkTypes::Edits, MESSAGES)?
            .into_iter()
            .map(|f| (f.entry.message, f.entry.body))
            .collect();
    let pinned = pins::pinned(chat)?;
    let mut messages = Vec::new();
    for (r, mut m) in page()? {
        if !chat
            .roster
            .in_at(r.action().author(), r.action().timestamp())
        {
            continue;
        }
        if let Some(body) = edits.get(r.action_address()) {
            m.body.clone_from(body);
        }
        messages.push((r, m));
    }
    let mut views = with_quotes(messages, &own::deleted()?)?;
    for v in &mut views {
        v.edited = edits.contains_key(&v.id);
        v.pinned = pinned.contains_key(&v.id);
    }
    views.extend(chat.leaves.iter().map(|f| MessageView {
        id: f.hash.clone(),
        author: f.author.clone(),
        body: String::new(),
        created_at: f.at,
        reply_to: None,
        kind: "system".into(),
        edited: false,
        pinned: false,
    }));
    views.sort_by_key(|v| v.created_at);
    let excess = views.len().saturating_sub(PAGE_MAX);
    views.drain(..excess);
    Ok(views)
}

/// The newest `PAGE_MAX` message records, oldest first.
fn page() -> ExternResult<Vec<(Record, Message)>> {
    let deleted = own::deleted()?;
    let mut found: BTreeMap<ActionHash, Record> = own::messages()?
        .into_iter()
        .filter(|r| !deleted.contains(r.action_address()))
        .map(|r| (r.action_address().clone(), r))
        .collect();

    let mut links = get_links(
        LinkQuery::try_new(anchor()?, LinkTypes::Messages)?,
        GetStrategy::default(),
    )?;
    links.sort_by_key(|l| std::cmp::Reverse(l.timestamp));
    let mut wanted = Vec::new();
    let mut seen = BTreeSet::new();
    for hash in links
        .into_iter()
        .filter_map(|l| l.target.into_action_hash())
    {
        if wanted.len() >= PAGE_MAX {
            break;
        }
        if !deleted.contains(&hash) && !found.contains_key(&hash) && seen.insert(hash.clone()) {
            wanted.push(hash);
        }
    }
    for r in records(&wanted)?.into_iter().flatten() {
        found.insert(r.action_address().clone(), r);
    }

    let mut messages = Vec::new();
    for r in found.into_values() {
        if let Some(m) = decode(&r)? {
            messages.push((r, m));
        }
    }
    messages.sort_by_key(|(r, _)| r.action().timestamp());
    let excess = messages.len().saturating_sub(PAGE_MAX);
    messages.drain(..excess);
    Ok(messages)
}

fn with_quotes(
    messages: Vec<(Record, Message)>,
    deleted: &BTreeSet<ActionHash>,
) -> ExternResult<Vec<MessageView>> {
    let mut quotes: BTreeMap<ActionHash, ReplyRef> = messages
        .iter()
        .map(|(r, m)| {
            let id = r.action_address().clone();
            let quote = ReplyRef {
                id: id.clone(),
                author: Some(r.action().author().clone()),
                preview: preview(&m.body),
                deleted: false,
            };
            (id, quote)
        })
        .collect();
    let outside: BTreeSet<ActionHash> = messages
        .iter()
        .filter_map(|(_, m)| m.reply_to.clone())
        .filter(|parent| !quotes.contains_key(parent))
        .collect();
    for parent in outside {
        let quote = quote_of(&parent, deleted)?;
        quotes.insert(parent, quote);
    }
    Ok(messages
        .into_iter()
        .map(|(r, m)| MessageView {
            id: r.action_address().clone(),
            author: r.action().author().clone(),
            created_at: r.action().timestamp(),
            reply_to: m.reply_to.and_then(|p| quotes.get(&p).cloned()),
            body: m.body,
            kind: "text".into(),
            edited: false,
            pinned: false,
        })
        .collect())
}

/// The quote for an original outside the page: older than it, or deleted.
fn quote_of(parent: &ActionHash, deleted: &BTreeSet<ActionHash>) -> ExternResult<ReplyRef> {
    let mut quote = ReplyRef {
        id: parent.clone(),
        author: None,
        preview: String::new(),
        deleted: deleted.contains(parent),
    };
    let Some(Details::Record(details)) = get_details(parent.clone(), GetOptions::default())? else {
        return Ok(quote);
    };
    quote.author = Some(details.record.action().author().clone());
    quote.deleted |= !details.deletes.is_empty();
    if !quote.deleted {
        if let Some(m) = decode(&details.record)? {
            quote.preview = preview(&m.body);
        }
    }
    Ok(quote)
}
