//! Feed scores, ported from `web-backend/app/services/feeds/ranking.py`.

use crate::Micros;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    /// Ranked by demand/opposition signals and members (projects, events).
    Signalled,
    /// Ranked by votes (threads, posts, help requests).
    Voted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    Trending,
    Recent,
    Top,
    Oldest,
}

impl Sort {
    /// web's `normalize_sort`: unknown values (and "popular") mean trending.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "recent" => Self::Recent,
            "top" => Self::Top,
            "oldest" => Self::Oldest,
            _ => Self::Trending,
        }
    }
}

/// What the index knows about one feed item.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemStats {
    pub support: u32,
    pub oppose: u32,
    pub votes: i32,
    pub comments: u32,
    /// Members plus "going" for events.
    pub members: u32,
    pub created_at: Micros,
    pub last_activity_at: Micros,
}

/// Type-aware trending score with caps and recency decay:
/// `decay = 1 / (1 + hours_since_last_activity / 36)`.
#[must_use]
pub fn trending_score(kind: ItemKind, stats: &ItemStats, now: Micros) -> f64 {
    let base = match kind {
        ItemKind::Signalled => {
            let total = stats.support + stats.oppose;
            let favorability = if total > 0 {
                f64::from(stats.support) / f64::from(total)
            } else {
                0.0
            };
            f64::from(stats.support.min(50) * 2)
                + (favorability * 20.0).min(20.0)
                + f64::from(stats.comments.min(30))
                + f64::from(stats.members.min(40))
        }
        ItemKind::Voted => f64::from(stats.votes.clamp(0, 40)) + f64::from(stats.comments.min(30)),
    };
    // Whole seconds fit an i32 for 68 years, which keeps the conversion exact.
    let secs = i32::try_from((now - stats.last_activity_at).max(0) / 1_000_000).unwrap_or(i32::MAX);
    let hours = f64::from(secs) / 3600.0;
    base / (1.0 + hours / 36.0)
}

/// Highest-rated first, no decay; negatives stay negative so opposed items sink.
#[must_use]
pub fn top_score(kind: ItemKind, stats: &ItemStats) -> i64 {
    match kind {
        ItemKind::Signalled => i64::from(stats.support) - i64::from(stats.oppose),
        ItemKind::Voted => i64::from(stats.votes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MICROS_PER_HOUR;

    const HOUR: Micros = MICROS_PER_HOUR;

    fn project(support: u32, oppose: u32, comments: u32, members: u32) -> ItemStats {
        ItemStats {
            support,
            oppose,
            comments,
            members,
            ..ItemStats::default()
        }
    }

    #[test]
    fn signalled_score_matches_web() {
        // least(10,50)*2 + least(10/12*20,20) + least(5,30) + least(3,40) = 20 + 16.67 + 5 + 3
        let score = trending_score(ItemKind::Signalled, &project(10, 2, 5, 3), 0);
        assert!((score - 44.666_666).abs() < 1e-3, "{score}");
    }

    #[test]
    fn caps_apply() {
        let score = trending_score(ItemKind::Signalled, &project(500, 0, 500, 500), 0);
        assert!((score - (100.0 + 20.0 + 30.0 + 40.0)).abs() < 1e-9);
        let voted = ItemStats {
            votes: -5,
            comments: 2,
            ..ItemStats::default()
        };
        assert!((trending_score(ItemKind::Voted, &voted, 0) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn decays_by_half_after_36_hours() {
        let stats = project(10, 0, 0, 0);
        let fresh = trending_score(ItemKind::Signalled, &stats, 0);
        let later = trending_score(ItemKind::Signalled, &stats, 36 * HOUR);
        assert!((later - fresh / 2.0).abs() < 1e-9);
    }

    #[test]
    fn top_is_net_signal() {
        assert_eq!(top_score(ItemKind::Signalled, &project(2, 5, 0, 0)), -3);
    }

    #[test]
    fn sort_parse_defaults_to_trending() {
        assert_eq!(Sort::parse("popular"), Sort::Trending);
        assert_eq!(Sort::parse(" Top "), Sort::Top);
        assert_eq!(Sort::parse("x"), Sort::Trending);
    }
}
