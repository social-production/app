//! The weekly active count (ADR 0095, `sp_active`): one `Active` link per person per week,
//! from `active.<week>` to its author, counted by any reader.

use std::collections::BTreeMap;

use hdk::prelude::*;
use identity_integrity::{active_path, LinkTypes};
use sp_active::{counted_week, week_of, week_start, weekly_active};

use crate::records::links;
use crate::standing::full_since;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct WeeklyActive {
    /// The week counted (`sp_active::week_of`): the last complete week.
    pub week: i64,
    pub count: u32,
}

/// Write this week's `Active` link unless the caller already did. Returns whether it wrote.
/// The UI calls it after a meaningful action (a vote, post, comment, role taken).
pub fn record() -> ExternResult<bool> {
    let now = sys_time()?.as_micros();
    if wrote_this_week(now)? {
        return Ok(false);
    }
    let me = agent_info()?.agent_initial_pubkey;
    let path = active_path(week_of(now)).path_entry_hash()?;
    create_link(path, me, LinkTypes::Active, ())?;
    Ok(true)
}

/// From the caller's own chain, so a second call right after the first sees it.
fn wrote_this_week(now: i64) -> ExternResult<bool> {
    let since = week_start(now);
    let active: ScopedLinkType = LinkTypes::Active.try_into()?;
    let filter = ChainQueryFilter::new().action_type(ActionType::CreateLink);
    Ok(query(filter)?.iter().any(|r| match &r.action().data {
        ActionData::CreateLink(l) => {
            r.action().timestamp().as_micros() >= since
                && l.zome_index == active.zome_index
                && l.link_type == active.zome_type
        }
        _ => false,
    }))
}

/// Counts weeks, remembering who had Full standing since when (one read per person).
#[derive(Default)]
pub struct Counter {
    since: BTreeMap<AgentPubKey, Option<i64>>,
    weeks: BTreeMap<i64, u32>,
}

impl Counter {
    /// Distinct Full members active in `week`.
    pub fn week(&mut self, week: i64) -> ExternResult<u32> {
        if let Some(n) = self.weeks.get(&week) {
            return Ok(*n);
        }
        let found: Vec<(AgentPubKey, i64)> =
            links(active_path(week).path_entry_hash()?, LinkTypes::Active)?
                .into_iter()
                .filter_map(|l| {
                    let who = l.target.into_agent_pub_key()?;
                    (who == l.author).then(|| (who, l.timestamp.as_micros()))
                })
                .collect();
        for (who, _) in &found {
            if !self.since.contains_key(who) {
                self.since.insert(who.clone(), full_since(who)?);
            }
        }
        let since = &self.since;
        let n = weekly_active(week, found, |who, at| {
            since.get(who).copied().flatten().is_some_and(|f| f < at)
        });
        self.weeks.insert(week, n);
        Ok(n)
    }
}

pub fn count_at(at: Option<Timestamp>) -> ExternResult<WeeklyActive> {
    let at = match at {
        Some(t) => t.as_micros(),
        None => sys_time()?.as_micros(),
    };
    let week = counted_week(at);
    Ok(WeeklyActive {
        week,
        count: Counter::default().week(week)?,
    })
}
