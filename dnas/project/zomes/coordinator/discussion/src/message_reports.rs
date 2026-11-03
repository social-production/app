//! Reports on a space's chat messages and comments (ADR 0071 part b): web's `comment` target.
//! The report entry and its votes are the same as a space's (`reports.rs`); only what is
//! reported and how its audience and popularity are counted differ. A message is measured
//! against its space's audience (web's comment electorate is its subject's), its own age, and
//! its own popularity: its voters plus twice the people who replied to it (web's
//! `load_target_engagement` for comments).

use std::collections::{BTreeMap, BTreeSet};

use discussion_integrity::{ChatMessage, LinkTypes};
use hdk::prelude::*;
use sp_core::chat::PAGE_MAX;
use sp_moderation::{check_report, popularity_score};

use crate::reports::{self, count, Engagement, ModerationView, ReportInput};
use crate::{anchor, guest, reads};

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageReportInput {
    pub message: ActionHash,
    pub reason: String,
    pub description: String,
}

/// One reported message's state (messages nobody reported are left out).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageModeration {
    pub id: ActionHash,
    pub moderation: ModerationView,
}

/// This space's message at `hash`, if it is one.
fn message_at(hash: &ActionHash) -> ExternResult<Option<(Record, ChatMessage)>> {
    let Some(record) = reads::record(hash)? else {
        return Ok(None);
    };
    let Some(message) = reads::decode(&record)? else {
        return Ok(None);
    };
    if message.space != anchor()? {
        return Ok(None);
    }
    Ok(Some((record, message)))
}

/// Who replied to each message on the chat's newest page, the author left out.
fn repliers() -> ExternResult<BTreeMap<ActionHash, BTreeSet<AgentPubKey>>> {
    let mut out: BTreeMap<ActionHash, BTreeSet<AgentPubKey>> = BTreeMap::new();
    for m in reads::list()? {
        if let Some(parent) = m.reply_to {
            out.entry(parent.id).or_default().insert(m.author);
        }
    }
    Ok(out)
}

fn voters(message: &ActionHash) -> ExternResult<u32> {
    let links = get_links(
        LinkQuery::try_new(message.clone(), LinkTypes::MessageVotes)?,
        GetStrategy::default(),
    )?;
    let people: BTreeSet<AgentPubKey> = links.into_iter().map(|l| l.author).collect();
    Ok(count(people.len()))
}

/// The space's audience, with the message's author, age and popularity.
fn engagement_of(
    record: &Record,
    space: &Engagement,
    replied: &BTreeMap<ActionHash, BTreeSet<AgentPubKey>>,
) -> ExternResult<Engagement> {
    let author = record.action().author().clone();
    let id = record.action_address();
    let replies = replied
        .get(id)
        .map_or(0, |who| count(who.iter().filter(|a| **a != author).count()));
    Ok(Engagement {
        author,
        created_at: record.action().timestamp().as_micros(),
        score: popularity_score(voters(id)?, replies),
        ..space.clone()
    })
}

/// One message's moderation state.
pub fn status(message: &ActionHash) -> ExternResult<ModerationView> {
    let Some((record, _)) = message_at(message)? else {
        return Err(guest("This message isn't here yet. Try again in a moment."));
    };
    let found = reports::reports_on(message)?;
    if found.is_empty() {
        return Ok(reports::visible());
    }
    let e = engagement_of(&record, &reports::engagement()?, &repliers()?)?;
    reports::status_of(&found, &e)
}

/// The reported ones among `ids`, each with its state. The space's audience and the replies
/// are read once, and only when something is reported.
pub fn of(ids: &[ActionHash]) -> ExternResult<Vec<MessageModeration>> {
    if ids.len() > PAGE_MAX {
        return Err(guest("Too many messages at once."));
    }
    let mut reported = Vec::new();
    for id in ids {
        let found = reports::reports_on(id)?;
        if !found.is_empty() {
            reported.push((id, found));
        }
    }
    if reported.is_empty() {
        return Ok(Vec::new());
    }
    let space = reports::engagement()?;
    let replied = repliers()?;
    let mut out = Vec::new();
    for (id, found) in reported {
        let Some((record, _)) = message_at(id)? else {
            continue;
        };
        let e = engagement_of(&record, &space, &replied)?;
        let moderation = reports::status_of(&found, &e)?;
        if moderation.report.is_some() || moderation.state != "visible" {
            out.push(MessageModeration {
                id: id.clone(),
                moderation,
            });
        }
    }
    Ok(out)
}

/// web's `submit_report` on a comment: one active report per message, never your own.
pub fn submit(input: &MessageReportInput) -> ExternResult<ModerationView> {
    check_report(&input.reason, &input.description).map_err(|r| guest(&r))?;
    let Some((record, _)) = message_at(&input.message)? else {
        return Err(guest("This message isn't here yet. Try again in a moment."));
    };
    if record.action().author() == &agent_info()?.agent_initial_pubkey {
        return Err(guest("You can't report yourself"));
    }
    if status(&input.message)?.report.is_some() {
        return Err(guest(reports::ALREADY_OPEN));
    }
    let report = ReportInput {
        reason: input.reason.clone(),
        description: input.description.clone(),
    };
    reports::open(input.message.clone(), &report)?;
    status(&input.message)
}
