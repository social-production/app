//! Discussion threads (web's threads, ADR 0041): anyone signed in starts one; never edited.
//! Lists are sorted like web's channel feeds (`sp_index`: trending, recent, top, oldest and a
//! time window). Reads merge the network with the caller's own chain.

use std::collections::{BTreeMap, BTreeSet};

use discussion_integrity::{decode_thread, EntryTypes, LinkTypes, Thread, UnitEntryTypes};
use hdk::prelude::*;
use sp_core::discussion::{check_thread, THREAD_PAGE_MAX};
use sp_index::{query, FeedQuery, ItemKind, ItemStats, Sort, Window};

use crate::comments::{comment_sets, CommentView};
use crate::votes::scores;
use crate::{guest, own, reads};

const THREADS: &str = "threads";

#[derive(Serialize, Deserialize, Debug)]
pub struct ThreadInput {
    pub title: String,
    pub body: String,
}

/// web's feed `sort` and `window` values; unknown ones mean trending and all time.
#[derive(Serialize, Deserialize, Debug)]
pub struct ThreadQuery {
    pub sort: String,
    pub window: String,
    pub utc_offset_minutes: i32,
}

/// One thread as a card or the thread page's header shows it.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThreadView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub title: String,
    pub body: String,
    pub created_at: Timestamp,
    /// web sets a thread's `last_activity_at` when it is created and never moves it.
    pub last_activity_at: Timestamp,
    pub vote_count: i32,
    pub my_vote: i8,
    pub comment_count: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ThreadDetail {
    pub thread: ThreadView,
    pub comments: Vec<CommentView>,
}

fn anchor() -> ExternResult<EntryHash> {
    Path::from(THREADS).path_entry_hash()
}

#[hdk_extern]
pub fn create_thread(input: ThreadInput) -> ExternResult<ActionHash> {
    let thread = Thread {
        title: input.title.trim().to_string(),
        body: input.body.trim().to_string(),
    };
    check_thread(&thread.title, &thread.body).map_err(|reason| guest(&reason))?;
    let hash = create_entry(EntryTypes::Thread(thread))?;
    create_link(anchor()?, hash.clone(), LinkTypes::Threads, ())?;
    Ok(hash)
}

/// The newest `THREAD_PAGE_MAX` threads, ordered as asked.
#[hdk_extern]
pub fn list_threads(q: ThreadQuery) -> ExternResult<Vec<ThreadView>> {
    let deleted = own::deleted()?;
    let mut found: BTreeMap<ActionHash, Record> = own::entries(UnitEntryTypes::Thread)?
        .into_iter()
        .filter(|r| !deleted.contains(r.action_address()))
        .map(|r| (r.action_address().clone(), r))
        .collect();
    let mut links = get_links(
        LinkQuery::try_new(anchor()?, LinkTypes::Threads)?,
        GetStrategy::default(),
    )?;
    links.sort_by_key(|l| std::cmp::Reverse(l.timestamp));
    let mut wanted = Vec::new();
    let mut seen = BTreeSet::new();
    for hash in links
        .into_iter()
        .filter_map(|l| l.target.into_action_hash())
    {
        if wanted.len() >= THREAD_PAGE_MAX {
            break;
        }
        if !deleted.contains(&hash) && !found.contains_key(&hash) && seen.insert(hash.clone()) {
            wanted.push(hash);
        }
    }
    for r in reads::records(&wanted)?.into_iter().flatten() {
        found.insert(r.action_address().clone(), r);
    }
    let mut threads = Vec::new();
    for r in found.into_values() {
        if let Some(t) = decode_thread(&r)? {
            threads.push((r, t));
        }
    }
    threads.sort_by_key(|(r, _)| std::cmp::Reverse(r.action().timestamp()));
    threads.truncate(THREAD_PAGE_MAX);
    let views = views(threads)?;
    sorted(views, &q)
}

/// One thread with its comments; an error when it can't be found.
#[hdk_extern]
pub fn get_thread(id: ActionHash) -> ExternResult<ThreadDetail> {
    let record = if own::deleted()?.contains(&id) {
        None
    } else {
        reads::record(&id)?
    };
    let Some((record, thread)) = record
        .map(|r| decode_thread(&r).map(|t| t.map(|t| (r, t))))
        .transpose()?
        .flatten()
    else {
        return Err(guest("Thread not found."));
    };
    let comments = crate::comments::list(&id)?;
    let thread = views(vec![(record, thread)])?
        .pop()
        .ok_or_else(|| guest("Thread not found."))?;
    Ok(ThreadDetail { thread, comments })
}

/// Cards with their vote and comment counts.
fn views(threads: Vec<(Record, Thread)>) -> ExternResult<Vec<ThreadView>> {
    let ids: Vec<ActionHash> = threads
        .iter()
        .map(|(r, _)| r.action_address().clone())
        .collect();
    let scores = scores(&ids)?;
    let comments = comment_sets(&ids)?;
    Ok(threads
        .into_iter()
        .zip(scores.into_iter().zip(comments))
        .map(|((r, t), (score, comments))| ThreadView {
            id: r.action_address().clone(),
            author: r.action().author().clone(),
            title: t.title,
            body: t.body,
            created_at: r.action().timestamp(),
            last_activity_at: r.action().timestamp(),
            vote_count: score.vote_count,
            my_vote: score.my_vote,
            comment_count: u32::try_from(comments.len()).unwrap_or(u32::MAX),
        })
        .collect())
}

/// web's `sort_order_columns` and time window, for vote-ranked items.
fn sorted(views: Vec<ThreadView>, q: &ThreadQuery) -> ExternResult<Vec<ThreadView>> {
    let stats: Vec<ItemStats> = views
        .iter()
        .map(|v| ItemStats {
            votes: v.vote_count,
            comments: v.comment_count,
            created_at: v.created_at.as_micros(),
            last_activity_at: v.last_activity_at.as_micros(),
            ..ItemStats::default()
        })
        .collect();
    let pairs: Vec<(&ThreadView, &ItemStats)> = views.iter().zip(&stats).collect();
    let feed = FeedQuery {
        sort: Sort::parse(&q.sort),
        window: Window::parse(&q.window),
        search: "",
        now: sys_time()?.as_micros(),
        utc_offset_minutes: q.utc_offset_minutes,
    };
    let ranked = query(
        &pairs,
        &feed,
        |_| ItemKind::Voted,
        |(_, s)| s,
        |_| Vec::new(),
    );
    Ok(ranked.into_iter().map(|(v, _)| (*v).clone()).collect())
}
