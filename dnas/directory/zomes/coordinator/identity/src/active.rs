//! The weekly active count (ADR 0095, rolling since ADR 0121, `sp_active`): `Active` links
//! from `active.day.<day>` to their author, at most one an hour, counted over the seven days
//! before any moment by any reader.

use std::collections::BTreeMap;

use hdk::prelude::*;
use identity_integrity::{active_path, LinkTypes};
use sp_active::{active_at, day_of, window_days, RECORD_GAP_US};

use crate::bot_review;
use crate::records::links;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct WeeklyActive {
    /// The moment counted back from (microseconds); the count covers the seven days before it.
    pub at: i64,
    pub count: u32,
}

/// Write an `Active` link unless the caller wrote one in the last hour. Returns whether it
/// wrote. The UI calls it after a meaningful action (a vote, post, comment, role taken).
pub fn record() -> ExternResult<bool> {
    let now = sys_time()?.as_micros();
    if wrote_since(now - RECORD_GAP_US)? {
        return Ok(false);
    }
    let me = agent_info()?.agent_initial_pubkey;
    let path = active_path(day_of(now)).path_entry_hash()?;
    create_link(path, me, LinkTypes::Active, ())?;
    Ok(true)
}

/// From the caller's own chain, so a second call right after the first sees it.
fn wrote_since(since: i64) -> ExternResult<bool> {
    let active: ScopedLinkType = LinkTypes::Active.try_into()?;
    let filter = ChainQueryFilter::new().action_type(ActionType::CreateLink);
    Ok(query(filter)?.iter().any(|r| match &r.action().data {
        ActionData::CreateLink(l) => {
            r.action().timestamp().as_micros() > since
                && l.zome_index == active.zome_index
                && l.link_type == active.zome_type
        }
        _ => false,
    }))
}

/// The `Active` links of a span of days, with each author's bot review read once, so any
/// moment in it can be counted without more reads (the board replays many moments).
pub struct Window {
    links: Vec<(AgentPubKey, i64)>,
    reviews: BTreeMap<AgentPubKey, Option<sp_bot_review::State<AgentPubKey>>>,
}

impl Window {
    /// Every link that counts at any moment in `from..=to`.
    pub fn load(from: i64, to: i64) -> ExternResult<Self> {
        let mut found = Vec::new();
        for day in *window_days(from).start()..=day_of(to) {
            for l in links(active_path(day).path_entry_hash()?, LinkTypes::Active)? {
                if let Some(who) = l.target.into_agent_pub_key() {
                    if who == l.author {
                        found.push((who, l.timestamp.as_micros()));
                    }
                }
            }
        }
        let mut reviews = BTreeMap::new();
        for (who, _) in &found {
            if !reviews.contains_key(who) {
                reviews.insert(who.clone(), bot_review::state(who)?);
            }
        }
        Ok(Self {
            links: found,
            reviews,
        })
    }

    /// Distinct people active in the seven days before `at`, leaving out accounts found to
    /// be bots when they wrote (ADR 0118: everyone votes, so everyone counts).
    pub fn count(&self, at: i64) -> u32 {
        active_at(at, self.links.iter().cloned(), |who, t| {
            !self
                .reviews
                .get(who)
                .and_then(Option::as_ref)
                .is_some_and(|s| s.paused_at(t))
        })
    }
}

pub fn count_at(at: Option<Timestamp>) -> ExternResult<WeeklyActive> {
    let at = match at {
        Some(t) => t.as_micros(),
        None => sys_time()?.as_micros(),
    };
    Ok(WeeklyActive {
        at,
        count: Window::load(at, at)?.count(at),
    })
}
