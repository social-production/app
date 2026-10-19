//! One feed page: filter by window and search text, then order like web's `sort_order_columns`.

use std::cmp::Ordering;

use crate::ranking::{top_score, trending_score, ItemKind, ItemStats, Sort};
use crate::search::matches;
use crate::window::{window_bounds, Window};
use crate::Micros;

#[derive(Clone, Copy, Debug)]
pub struct FeedQuery<'a> {
    pub sort: Sort,
    pub window: Window,
    pub search: &'a str,
    pub now: Micros,
    pub utc_offset_minutes: i32,
}

/// Returns the items that pass the window and search, best first.
/// `text` gives the searchable fields of an item (title, description, tags…).
pub fn query<'i, T>(
    items: &'i [T],
    q: &FeedQuery<'_>,
    kind: impl Fn(&T) -> ItemKind,
    stats: impl Fn(&T) -> &ItemStats,
    text: impl Fn(&T) -> Vec<&str>,
) -> Vec<&'i T> {
    let bounds = window_bounds(q.window, q.now, q.utc_offset_minutes);
    let mut kept: Vec<(&T, f64, i64)> = items
        .iter()
        .filter(|item| {
            let at = stats(item).last_activity_at;
            bounds.is_none_or(|(start, end)| at >= start && at < end)
        })
        .filter(|item| matches(q.search, &text(item)))
        .map(|item| {
            let s = stats(item);
            (
                item,
                trending_score(kind(item), s, q.now),
                top_score(kind(item), s),
            )
        })
        .collect();
    kept.sort_by(|a, b| order(q.sort, stats(a.0), stats(b.0), (a.1, a.2), (b.1, b.2)));
    kept.into_iter().map(|(item, _, _)| item).collect()
}

fn order(sort: Sort, a: &ItemStats, b: &ItemStats, sa: (f64, i64), sb: (f64, i64)) -> Ordering {
    let recent = || b.last_activity_at.cmp(&a.last_activity_at);
    match sort {
        Sort::Recent => recent().then(b.created_at.cmp(&a.created_at)),
        Sort::Oldest => a
            .last_activity_at
            .cmp(&b.last_activity_at)
            .then(a.created_at.cmp(&b.created_at)),
        Sort::Top => sb.1.cmp(&sa.1).then_with(recent),
        Sort::Trending => sb.0.total_cmp(&sa.0).then_with(recent),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MICROS_PER_HOUR as HOUR;

    struct P {
        title: &'static str,
        stats: ItemStats,
    }

    fn p(title: &'static str, support: u32, oppose: u32, hours_ago: i64) -> P {
        let at = 100 * HOUR - hours_ago * HOUR;
        P {
            title,
            stats: ItemStats {
                support,
                oppose,
                created_at: at,
                last_activity_at: at,
                ..ItemStats::default()
            },
        }
    }

    fn titles(sort: Sort, window: Window, search: &str, items: &[P]) -> Vec<&'static str> {
        let q = FeedQuery {
            sort,
            window,
            search,
            now: 100 * HOUR,
            utc_offset_minutes: 0,
        };
        query(
            items,
            &q,
            |_| ItemKind::Signalled,
            |i| &i.stats,
            |i| vec![i.title],
        )
        .into_iter()
        .map(|i| i.title)
        .collect()
    }

    #[test]
    fn sorts_like_web() {
        let items = [
            p("old popular", 20, 0, 90),
            p("fresh", 3, 0, 1),
            p("opposed", 1, 9, 2),
        ];
        assert_eq!(
            titles(Sort::Recent, Window::All, "", &items),
            ["fresh", "opposed", "old popular"]
        );
        assert_eq!(
            titles(Sort::Oldest, Window::All, "", &items),
            ["old popular", "opposed", "fresh"]
        );
        assert_eq!(
            titles(Sort::Top, Window::All, "", &items),
            ["old popular", "fresh", "opposed"]
        );
        // 90 hours of decay (÷3.5) outweighs 20 supports: 60 / 3.5 ≈ 17 < 26 / 1.03 ≈ 25.
        assert_eq!(titles(Sort::Trending, Window::All, "", &items)[0], "fresh");
    }

    #[test]
    fn window_and_search_filter() {
        // now = hour 100 = day 4, 04:00 UTC; "today" keeps the last 4 hours.
        let items = [
            p("garden", 1, 0, 1),
            p("garden shed", 1, 0, 10),
            p("bikes", 1, 0, 2),
        ];
        assert_eq!(
            titles(Sort::Recent, Window::Today, "", &items),
            ["garden", "bikes"]
        );
        assert_eq!(
            titles(Sort::Recent, Window::All, "GARDEN", &items),
            ["garden", "garden shed"]
        );
    }
}
