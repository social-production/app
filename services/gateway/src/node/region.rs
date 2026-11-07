//! web's Region feed (`get_region_feed` in web-backend's `feeds/region.py`, M6 phase 4c,
//! ADR 0084) over this node's index: public cards with a physical place within the radius,
//! nearest first, plus project activities with their own place, plus (when asked) online
//! items appended after them. The index holds public spaces only, so web's private-community
//! rule holds by construction.

use serde::Serialize;
use sp_index::{window_bounds, FeedQuery, Sort, Window};

use super::markers::ActivityPlace;
use super::ProjectCard;

/// web fetches this many ranked rows before the distance filter.
const RANKED_MAX: usize = 300;
const PROJECT_MODES: [&str; 3] = ["productive", "collective-service", "personal-service"];
/// web's `_online_region_items` also counts these labels as online.
const ONLINE_LABELS: [&str; 3] = ["online", "remote", "virtual"];

/// web's `/feeds/region` parameters, checked by the route.
pub struct RegionQuery<'a> {
    pub center: (f64, f64),
    pub radius_km: f64,
    pub sort: Sort,
    pub window: Window,
    /// `all`, `projects`, `events`, `help_requests` or `threads` (which is always empty).
    pub filter: &'a str,
    pub include_online: bool,
    pub utc_offset_minutes: i32,
    pub limit: usize,
    pub offset: usize,
    /// Holochain microseconds.
    pub now: i64,
}

/// A feed card with web's `distance_km` and `is_online` (null and true for online items).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionCard {
    #[serde(flatten)]
    pub card: ProjectCard,
    pub distance_km: Option<f64>,
    pub is_online: bool,
}

/// web's `project_activity` item (`PublicProjectActivityItem` on web's frontend).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionActivity {
    pub kind: &'static str,
    pub id: String,
    pub parent_id: String,
    pub href: String,
    pub title: String,
    pub parent_title: String,
    pub project_mode: String,
    pub location_label: String,
    pub scheduled_at: i64,
    pub ends_at: i64,
    pub created_at: i64,
    pub last_activity_at: i64,
    pub distance_km: f64,
    pub is_online: bool,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum RegionItem {
    Card(Box<RegionCard>),
    Activity(Box<RegionActivity>),
}

impl RegionItem {
    /// web's order: distance, then the item's time (`scheduled_at`, else `last_activity_at`).
    fn key(&self) -> (f64, i64) {
        match self {
            RegionItem::Card(c) => (c.distance_km.unwrap_or(f64::MAX), when(&c.card)),
            RegionItem::Activity(a) => (a.distance_km, a.scheduled_at),
        }
    }
}

/// web's `filter_entity_type` for a card: projects, events and help requests have places.
fn wanted(card: &ProjectCard, filter: &str) -> bool {
    let kind = match card.mode.as_str() {
        "event" => "events",
        "help-request" => "help_requests",
        m if PROJECT_MODES.contains(&m) => "projects",
        _ => return false,
    };
    card.moderation.as_deref() != Some(super::index::REMOVED) && (filter == "all" || filter == kind)
}

/// web's `scheduled_at` column: an event's start, a help request's time, a project's earliest
/// activity (`_next_project_activity_scheduled_at`).
fn scheduled(card: &ProjectCard) -> Option<i64> {
    match card.mode.as_str() {
        "event" => card.scheduled_at,
        "help-request" => card.help.as_ref().and_then(|h| h.needed_at),
        _ => card.activities.iter().map(|a| a.starts_at).min(),
    }
}

fn when(card: &ProjectCard) -> i64 {
    scheduled(card).unwrap_or(card.last_activity_at)
}

/// web's `apply_schedule_window_filter`: outside "all", only items scheduled in the window.
fn in_window(at: Option<i64>, bounds: Option<(i64, i64)>) -> bool {
    bounds.is_none_or(|(from, to)| at.is_some_and(|t| t >= from && t < to))
}

fn round_km(distance: f64) -> f64 {
    (distance * 100.0).round() / 100.0
}

/// Cards ranked by the feed's sort (web orders by it before cutting to 300 rows).
fn ranked<'c>(cards: &[&'c ProjectCard], q: &RegionQuery, sort: Sort) -> Vec<&'c ProjectCard> {
    let feed = FeedQuery {
        sort,
        window: Window::All,
        search: "",
        now: q.now,
        utc_offset_minutes: q.utc_offset_minutes,
    };
    super::index::rank(cards.to_vec(), &feed)
}

fn nearby(cards: &[&ProjectCard], q: &RegionQuery, bounds: Option<(i64, i64)>) -> Vec<RegionItem> {
    let kept: Vec<&ProjectCard> = cards
        .iter()
        .copied()
        .filter(|c| in_window(scheduled(c), bounds))
        .collect();
    ranked(&kept, q, q.sort)
        .into_iter()
        .take(RANKED_MAX)
        .filter_map(|c| {
            let p = c.place.as_ref()?;
            let distance = sp_place::distance_km(q.center, (p.latitude, p.longitude));
            (distance <= q.radius_km).then(|| {
                RegionItem::Card(Box::new(RegionCard {
                    card: c.clone(),
                    distance_km: Some(round_km(distance)),
                    is_online: false,
                }))
            })
        })
        .collect()
}

/// web's `_region_project_activity_feed_items`: upcoming activities of projects (not events)
/// at their own physical place (web joins the activity's location, not its parent's), in the
/// window; a full personal-service activity is left out.
fn activities(
    cards: &[&ProjectCard],
    q: &RegionQuery,
    bounds: Option<(i64, i64)>,
) -> Vec<RegionItem> {
    let mut out = Vec::new();
    for c in cards
        .iter()
        .filter(|c| PROJECT_MODES.contains(&c.mode.as_str()))
    {
        for a in &c.activities {
            let ActivityPlace::Own(p) = &a.place else {
                continue;
            };
            let full = c.mode == "personal-service" && a.minimum > 0 && a.committed >= a.minimum;
            if full || a.starts_at < q.now || !in_window(Some(a.starts_at), bounds) {
                continue;
            }
            let distance = sp_place::distance_km(q.center, (p.latitude, p.longitude));
            if distance > q.radius_km {
                continue;
            }
            out.push(RegionItem::Activity(Box::new(RegionActivity {
                kind: "project-activity",
                id: a.id.clone(),
                parent_id: c.id.clone(),
                href: format!("/projects/{}?activity={}", c.id, a.id),
                title: a.title.clone(),
                parent_title: c.title.clone(),
                project_mode: c.mode.clone(),
                location_label: a.label.clone(),
                scheduled_at: a.starts_at,
                ends_at: a.ends_at,
                created_at: a.starts_at,
                last_activity_at: a.starts_at,
                distance_km: round_km(distance),
                is_online: false,
            })));
        }
    }
    out
}

/// web's `_online_region_items`: the most recent online items, up to the page size.
fn online(cards: &[&ProjectCard], q: &RegionQuery, bounds: Option<(i64, i64)>) -> Vec<RegionItem> {
    let kept: Vec<&ProjectCard> = cards
        .iter()
        .copied()
        .filter(|c| in_window(scheduled(c), bounds))
        .collect();
    ranked(&kept, q, Sort::Recent)
        .into_iter()
        .take(q.limit * 3)
        .filter(|c| {
            let label = c.location_label.trim().to_lowercase();
            c.online || ONLINE_LABELS.contains(&label.as_str())
        })
        .take(q.limit)
        .map(|c| {
            RegionItem::Card(Box::new(RegionCard {
                card: c.clone(),
                distance_km: None,
                is_online: true,
            }))
        })
        .collect()
}

/// One page of web's Region feed and its `total`.
pub fn region(all: &[ProjectCard], q: &RegionQuery) -> (Vec<RegionItem>, usize) {
    if q.filter == "threads" {
        return (Vec::new(), 0);
    }
    let cards: Vec<&ProjectCard> = all.iter().filter(|c| wanted(c, q.filter)).collect();
    let bounds = window_bounds(q.window, q.now, q.utc_offset_minutes);
    let mut near = nearby(&cards, q, bounds);
    if q.filter == "all" || q.filter == "projects" {
        near.extend(activities(&cards, q, bounds));
    }
    near.sort_by(|a, b| {
        let (ka, kb) = (a.key(), b.key());
        ka.0.total_cmp(&kb.0).then(ka.1.cmp(&kb.1))
    });
    let mut total = near.len();
    let mut page: Vec<RegionItem> = near.into_iter().skip(q.offset).take(q.limit).collect();
    if q.include_online {
        // Appended after the page, never ranked among the pins (web).
        let extra = online(&cards, q, bounds);
        total += extra.len();
        page.extend(extra);
    }
    (page, total)
}

#[cfg(test)]
#[path = "region_tests.rs"]
mod tests;
