//! A space's next planned activity on its feed card (owner, 2026-11-24: "all content that has a
//! next activity planned should show" it, as the Meetup mockup's card does).

use serde::Serialize;

use super::index::ProjectCard;
use super::markers::CardActivity;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextActivity {
    pub id: String,
    pub title: String,
    /// Holochain microseconds.
    pub starts_at: i64,
    pub ends_at: i64,
    /// People in its roles, and the roles' required count (0: no roles).
    pub committed: u32,
    pub minimum: u32,
    /// Activities after this one that haven't ended either.
    pub more: u32,
}

/// The soonest activity not over at `now`, with how many more are still to come.
#[must_use]
pub fn next_of(activities: &[CardActivity], now: i64) -> Option<NextActivity> {
    let mut open: Vec<&CardActivity> = activities.iter().filter(|a| a.ends_at > now).collect();
    open.sort_by_key(|a| (a.starts_at, a.id.clone()));
    let first = open.first()?;
    Some(NextActivity {
        id: first.id.clone(),
        title: first.title.clone(),
        starts_at: first.starts_at,
        ends_at: first.ends_at,
        committed: first.committed,
        minimum: first.minimum,
        more: u32::try_from(open.len() - 1).unwrap_or(u32::MAX),
    })
}

/// A copy of `card` to serve at `now`, with its next activity.
#[must_use]
pub fn served(card: &ProjectCard, now: i64) -> ProjectCard {
    let mut out = card.clone();
    out.next_activity = next_of(&card.activities, now);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markers::ActivityPlace;

    fn activity(id: &str, starts_at: i64, ends_at: i64) -> CardActivity {
        CardActivity {
            id: id.into(),
            title: format!("Session {id}"),
            starts_at,
            ends_at,
            label: String::new(),
            place: ActivityPlace::Parent,
            committed: 2,
            minimum: 4,
        }
    }

    #[test]
    fn the_soonest_unfinished_activity_is_next() {
        let all = [
            activity("c", 300, 400),
            activity("a", 50, 90),
            activity("b", 100, 200),
        ];
        let next = next_of(&all, 95).expect("one is still to come");
        assert_eq!((next.id.as_str(), next.more), ("b", 1));
        // One that has started but not ended is still the next.
        assert_eq!(next_of(&all, 150).map(|n| n.id), Some("b".into()));
        assert!(next_of(&all, 400).is_none(), "all over");
        assert!(next_of(&[], 0).is_none());
    }
}
