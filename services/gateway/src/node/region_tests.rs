//! Tests for web's Region feed (`region.rs`).

use super::*;
use crate::node::cards::HelpCard;
use crate::node::markers::{CardActivity, CardPlace};

const FITZROY: (f64, f64) = (-37.8, 144.98);
const DAY: i64 = 86_400_000_000;
const NOW: i64 = 1_000 * DAY;

fn at(spot: (f64, f64)) -> Option<CardPlace> {
    Some(CardPlace {
        latitude: spot.0,
        longitude: spot.1,
        precision: "approximate".into(),
    })
}

fn own_place() -> CardPlace {
    CardPlace {
        latitude: FITZROY.0,
        longitude: FITZROY.1,
        precision: "approximate".into(),
    }
}

fn card(id: &str, mode: &str, place: Option<CardPlace>) -> ProjectCard {
    ProjectCard {
        id: id.into(),
        network_seed: String::new(),
        creator: String::new(),
        creator_name: String::new(),
        title: id.into(),
        summary: String::new(),
        mode: mode.into(),
        subtype: "standard".into(),
        location_label: "Somewhere".into(),
        phase: String::new(),
        tags: Vec::new(),
        members: 1,
        demand: 0,
        opposition: 0,
        created_at: 0,
        last_activity_at: 0,
        latest_update: None,
        comment_count: 0,
        help: None,
        scheduled_at: None,
        moderation: None,
        place,
        activities: Vec::new(),
        online: false,
    }
}

fn activity(id: &str, starts_at: i64, place: ActivityPlace) -> CardActivity {
    CardActivity {
        id: id.into(),
        title: id.into(),
        starts_at,
        ends_at: starts_at + DAY / 24,
        label: format!("{id} hall"),
        place,
        committed: 0,
        minimum: 0,
    }
}

fn query(filter: &str) -> RegionQuery<'_> {
    RegionQuery {
        center: FITZROY,
        radius_km: 25.0,
        sort: Sort::Trending,
        window: Window::All,
        filter,
        include_online: false,
        utc_offset_minutes: 0,
        limit: 20,
        offset: 0,
        now: NOW,
    }
}

fn ids(items: &[RegionItem]) -> Vec<String> {
    items
        .iter()
        .map(|i| match i {
            RegionItem::Card(c) => c.card.id.clone(),
            RegionItem::Activity(a) => a.id.clone(),
        })
        .collect()
}

#[test]
fn nearest_first_within_the_radius() {
    let cards = [
        card("brunswick", "productive", at((-37.77, 144.96))),
        card("fitzroy", "productive", at(FITZROY)),
        card("sydney", "productive", at((-33.87, 151.21))),
        card("online", "productive", None),
        card("thread", "thread", at(FITZROY)),
    ];
    let (items, total) = region(&cards, &query("all"));
    assert_eq!(ids(&items), ["fitzroy", "brunswick"]);
    assert_eq!(total, 2);
    let RegionItem::Card(first) = &items[0] else {
        panic!("a card")
    };
    assert_eq!(first.distance_km, Some(0.0));
    assert!(!first.is_online);
}

#[test]
fn filters_by_kind_and_threads_are_empty() {
    let cards = [
        card("p", "productive", at(FITZROY)),
        card("e", "event", at(FITZROY)),
        card("h", "help-request", at(FITZROY)),
    ];
    assert_eq!(ids(&region(&cards, &query("events")).0), ["e"]);
    assert_eq!(ids(&region(&cards, &query("help_requests")).0), ["h"]);
    let (none, total) = region(&cards, &query("threads"));
    assert!(none.is_empty());
    assert_eq!(total, 0);
}

#[test]
fn removed_items_are_left_out() {
    let mut gone = card("gone", "productive", at(FITZROY));
    gone.moderation = Some(super::super::index::REMOVED.into());
    assert!(region(&[gone], &query("all")).0.is_empty());
}

#[test]
fn project_activities_at_their_own_place_join_in() {
    let mut p = card("garden", "productive", at((-37.77, 144.96)));
    p.activities = vec![
        activity("own", NOW + DAY, ActivityPlace::Own(own_place())),
        // web joins the activity's own location only.
        activity("parent", NOW + DAY, ActivityPlace::Parent),
        activity("past", NOW - DAY, ActivityPlace::Own(own_place())),
    ];
    let (items, total) = region(&[p.clone()], &query("all"));
    assert_eq!(ids(&items), ["own", "garden"]);
    assert_eq!(total, 2);
    let RegionItem::Activity(a) = &items[0] else {
        panic!("an activity")
    };
    assert_eq!(a.kind, "project-activity");
    assert_eq!(a.href, "/projects/garden?activity=own");
    assert_eq!(a.location_label, "own hall");
    // Not under the events filter, and not for an event's activities.
    assert!(region(&[p.clone()], &query("events")).0.is_empty());
    p.mode = "event".into();
    assert_eq!(ids(&region(&[p], &query("all")).0), ["garden"]);
}

#[test]
fn a_full_personal_service_activity_is_left_out() {
    let mut s = card("care", "personal-service", None);
    let mut full = activity("full", NOW + DAY, ActivityPlace::Own(own_place()));
    (full.committed, full.minimum) = (2, 2);
    s.activities = vec![full.clone()];
    assert!(region(&[s.clone()], &query("all")).0.is_empty());
    s.mode = "collective-service".into();
    s.activities = vec![full];
    assert_eq!(ids(&region(&[s], &query("all")).0), ["full"]);
}

#[test]
fn a_window_keeps_items_scheduled_in_it() {
    let mut soon = card("soon", "event", at(FITZROY));
    soon.scheduled_at = Some(NOW + DAY / 48);
    let mut later = card("later", "event", at(FITZROY));
    later.scheduled_at = Some(NOW + 40 * DAY);
    let untimed = card("untimed", "productive", at(FITZROY));
    let mut q = query("all");
    q.window = Window::Today;
    assert_eq!(ids(&region(&[soon, later, untimed], &q).0), ["soon"]);
}

#[test]
fn same_distance_orders_by_time() {
    let mut a = card("a", "event", at(FITZROY));
    a.scheduled_at = Some(NOW + 2 * DAY);
    let mut b = card("b", "event", at(FITZROY));
    b.scheduled_at = Some(NOW + DAY);
    assert_eq!(ids(&region(&[a, b], &query("all")).0), ["b", "a"]);
}

#[test]
fn pages_then_appends_online_items() {
    let cards: Vec<ProjectCard> = (0..3)
        .map(|i| card(&format!("near{i}"), "productive", at(FITZROY)))
        .chain([
            ProjectCard {
                online: true,
                ..card("call", "event", None)
            },
            ProjectCard {
                location_label: "Remote".into(),
                ..card("remote", "help-request", None)
            },
        ])
        .collect();
    let mut q = query("all");
    (q.limit, q.offset) = (2, 2);
    let (items, total) = region(&cards, &q);
    assert_eq!(total, 3);
    assert_eq!(items.len(), 1);
    q.include_online = true;
    let (items, total) = region(&cards, &q);
    assert_eq!(total, 5);
    assert_eq!(items.len(), 3);
    let RegionItem::Card(last) = &items[2] else {
        panic!("a card")
    };
    assert!(last.is_online);
    assert_eq!(last.distance_km, None);
    // Online items follow the filter too.
    q.filter = "events";
    let (items, _) = region(&cards, &q);
    assert_eq!(ids(&items), ["call"]);
}

#[test]
fn cards_send_distance_and_online_beside_the_card() -> Result<(), serde_json::Error> {
    let mut c = card("fitzroy", "event", at(FITZROY));
    c.help = Some(HelpCard::default());
    let (items, _) = region(&[c], &query("all"));
    let json = serde_json::to_value(&items[0])?;
    assert_eq!(json["id"], "fitzroy");
    assert_eq!(json["distanceKm"], 0.0);
    assert_eq!(json["isOnline"], false);
    assert!(json.get("activities").is_none());
    Ok(())
}
