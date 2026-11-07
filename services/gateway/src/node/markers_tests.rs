//! Tests for web's map markers (`markers.rs`).

use super::*;
use crate::node::cards::HelpCard;

const FITZROY: (f64, f64) = (-37.8, 144.98);

fn card(id: &str, mode: &str, place: Option<(f64, f64)>) -> ProjectCard {
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
        place: place.map(|(latitude, longitude)| CardPlace {
            latitude,
            longitude,
            precision: "approximate".into(),
        }),
    }
}

fn query(filter: &str) -> MarkerQuery<'_> {
    MarkerQuery {
        center: FITZROY,
        radius_km: 25.0,
        filter,
        upcoming_only: true,
        limit: 200,
        now: 1_000,
    }
}

#[test]
fn pins_nearby_physical_places_only() {
    let cards = [
        card("near", "productive", Some((-37.81, 144.96))),
        card("far", "productive", Some((-33.87, 151.21))), // Sydney
        card("online", "productive", None),
        card("thread", "thread", Some(FITZROY)),
    ];
    let ids: Vec<String> = markers(&cards, &query("all"))
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(ids, ["near"]);
    let m = &markers(&cards, &query("all"))[0];
    assert_eq!(
        (m.entity_type, m.href.as_str()),
        ("project", "/projects/near")
    );
    assert_eq!(m.subtitle.as_deref(), Some("Proposal"));
    assert!(
        m.distance_km > 1.0 && m.distance_km < 3.0,
        "{}",
        m.distance_km
    );
}

#[test]
fn timed_items_need_a_time_and_upcoming_ones_come_first() {
    let mut later = card("later", "event", Some(FITZROY));
    later.scheduled_at = Some(9_000);
    let mut soon = card("soon", "help-request", Some(FITZROY));
    soon.help = Some(HelpCard {
        needed_at: Some(2_000),
        ends_at: Some(3_000),
        signup_count: 1,
        slots_needed: 2,
    });
    let mut past = card("past", "event", Some(FITZROY));
    past.scheduled_at = Some(500);
    let unscheduled = card("unscheduled", "event", Some(FITZROY));
    let project = card("project", "collective-service", Some(FITZROY));
    let cards = [later, soon, past, unscheduled, project];
    let ids: Vec<String> = markers(&cards, &query("all"))
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(ids, ["soon", "later", "project"]);
    let all = MarkerQuery {
        upcoming_only: false,
        ..query("events")
    };
    let ids: Vec<String> = markers(&cards, &all).into_iter().map(|m| m.id).collect();
    assert_eq!(ids, ["past", "later"]);
    let help = &markers(&cards, &query("help_requests"))[0];
    assert_eq!((help.signup_count, help.slots_needed), (Some(1), Some(2)));
    assert_eq!(help.href, "/help-requests/soon");
}

#[test]
fn removed_items_and_online_places_have_no_pin() {
    let mut removed = card("removed", "productive", Some(FITZROY));
    removed.moderation = Some(crate::node::index::REMOVED.into());
    assert!(markers(&[removed], &query("all")).is_empty());
    let online = PlaceMirror {
        latitude: None,
        longitude: None,
        precision: "approximate".into(),
        online: true,
    };
    assert_eq!(online.physical(), None);
}
