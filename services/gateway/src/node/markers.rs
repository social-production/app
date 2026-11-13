//! Places on feed cards and web's map markers (M6 phase 4, ADR 0082 and 0083): web-backend's
//! `get_map_markers` (`feeds/region.py`) over this node's index. The index holds public spaces
//! only, so private places never reach a marker (web's visibility rule holds by construction).

use serde::{Deserialize, Serialize};

use super::ProjectCard;

/// Mirrors `project_integrity::Place` (the fields the map needs).
#[derive(Clone, Debug, Deserialize)]
pub struct PlaceMirror {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub precision: String,
    #[serde(default)]
    pub online: bool,
}

impl PlaceMirror {
    /// A pin for a physical place with coordinates; none for an online one.
    pub fn physical(&self) -> Option<CardPlace> {
        match (self.online, self.latitude, self.longitude) {
            (false, Some(latitude), Some(longitude)) => Some(CardPlace {
                latitude,
                longitude,
                precision: self.precision.clone(),
            }),
            _ => None,
        }
    }
}

/// A card's coordinates, as rounded by its writer (`sp_place`).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CardPlace {
    pub latitude: f64,
    pub longitude: f64,
    pub precision: String,
}

/// Where an activity is on the map: web joins the activity's location, or its parent's when it
/// has none (`coalesce`); an online one has no pin.
#[derive(Clone, Debug, PartialEq)]
pub enum ActivityPlace {
    Own(CardPlace),
    Parent,
    Elsewhere,
}

/// What the index keeps of an activity (ADR 0083).
#[derive(Clone, Debug)]
pub struct CardActivity {
    pub id: String,
    pub title: String,
    pub starts_at: i64,
    pub ends_at: i64,
    pub label: String,
    pub place: ActivityPlace,
    /// People in its roles, and the sum of the roles' required counts (web's
    /// `committed_count`, `minimum_participants`).
    pub committed: u32,
    pub minimum: u32,
}

/// web's `/feeds/map-markers` parameters.
pub struct MarkerQuery<'a> {
    pub center: (f64, f64),
    pub radius_km: f64,
    /// `all`, `projects`, `events` or `help_requests`.
    pub filter: &'a str,
    pub upcoming_only: bool,
    pub limit: usize,
    /// Holochain microseconds.
    pub now: i64,
    /// `[from, to)` from the calendar window and the dates (web's `_within_window`); `None`
    /// keeps every time, but then an item without one too.
    pub span: Option<(i64, i64)>,
}

impl MarkerQuery<'_> {
    /// web's `_include_item` for a timed item.
    fn includes(&self, at: i64) -> bool {
        let in_span = self.span.is_none_or(|(from, to)| at >= from && at < to);
        in_span && (!self.upcoming_only || at >= self.now)
    }
}

/// web's `MapMarkerOut` (snake case, as web's frontend reads it).
#[derive(Debug, Serialize)]
pub struct Marker {
    pub id: String,
    pub entity_type: &'static str,
    pub activity_source: Option<&'static str>,
    pub project_mode: Option<String>,
    pub title: String,
    pub parent_id: Option<String>,
    pub parent_title: Option<String>,
    pub subtitle: Option<String>,
    pub href: String,
    pub latitude: f64,
    pub longitude: f64,
    pub precision: String,
    pub display_label: String,
    pub distance_km: f64,
    /// Microseconds (the app's other index times are too).
    pub scheduled_at: Option<i64>,
    pub ends_at: Option<i64>,
    pub signup_count: Option<u32>,
    pub slots_needed: Option<u32>,
    pub committed_count: Option<u32>,
    pub minimum_participants: Option<u32>,
}

/// The modes that are projects on the map and in Region; a `proposal` is one before its plan
/// gives it a type (ADR 0059).
const PROJECT_MODES: [&str; 4] = [
    "proposal",
    "productive",
    "collective-service",
    "personal-service",
];

/// What a card is on the map, or `None` when web shows no pin for it.
fn kind_of(card: &ProjectCard, filter: &str) -> Option<&'static str> {
    let kind = match card.mode.as_str() {
        "event" => "event",
        "help-request" => "help_request",
        // web's `_project_pin_eligible`; a personal service's requests are on by default in
        // the app (ADR 0077), and the index doesn't read the setting.
        m if PROJECT_MODES.contains(&m) => "project",
        _ => return None,
    };
    let wanted = match filter {
        "projects" => kind == "project",
        "events" => kind == "event",
        "help_requests" => kind == "help_request",
        _ => true,
    };
    wanted.then_some(kind)
}

/// web's `_project_pin_subtitle`.
fn subtitle(card: &ProjectCard, q: &MarkerQuery) -> String {
    let phase = card.phase.trim();
    let open_spots = card
        .activities
        .iter()
        .any(|a| (!q.upcoming_only || a.starts_at >= q.now) && a.committed < a.minimum);
    match card.mode.as_str() {
        "productive" | "proposal" if phase.is_empty() => "Proposal".into(),
        "productive" | "proposal" => phase.into(),
        "personal-service" => "Accepting requests".into(),
        _ if open_spots => "Open spots".into(),
        _ => phase.into(),
    }
}

fn path_of(kind: &str) -> &'static str {
    match kind {
        "event" => "events",
        "help_request" => "help-requests",
        _ => "projects",
    }
}

fn round_km(distance: f64) -> f64 {
    (distance * 100.0).round() / 100.0
}

/// The card's own pin, if it has one and its time (if it is timed) is wanted.
fn card_marker(card: &ProjectCard, kind: &'static str, q: &MarkerQuery) -> Option<Marker> {
    let place = card.place.as_ref()?;
    let distance = sp_place::distance_km(q.center, (place.latitude, place.longitude));
    if distance > q.radius_km {
        return None;
    }
    let help = card.help.as_ref();
    let scheduled_at = match kind {
        "event" => Some(card.scheduled_at?),
        "help_request" => Some(help?.needed_at?),
        _ => None,
    };
    if scheduled_at.is_some_and(|t| !q.includes(t)) {
        return None;
    }
    Some(Marker {
        id: card.id.clone(),
        entity_type: kind,
        activity_source: None,
        project_mode: (kind == "project").then(|| card.mode.clone()),
        title: card.title.clone(),
        parent_id: None,
        parent_title: None,
        subtitle: (kind == "project").then(|| subtitle(card, q)),
        href: format!("/{}/{}", path_of(kind), card.id),
        latitude: place.latitude,
        longitude: place.longitude,
        precision: place.precision.clone(),
        display_label: card.location_label.clone(),
        distance_km: round_km(distance),
        scheduled_at,
        ends_at: help.and_then(|h| h.ends_at),
        signup_count: help.map(|h| h.signup_count),
        slots_needed: help.map(|h| h.slots_needed),
        committed_count: None,
        minimum_participants: None,
    })
}

/// web's event and project activity markers; a full activity of a service has none.
fn activity_markers(card: &ProjectCard, kind: &'static str, q: &MarkerQuery) -> Vec<Marker> {
    let source = match kind {
        "event" => "event",
        "project" => "project",
        _ => return Vec::new(), // a help request's one activity is its own pin
    };
    let service = card.mode.ends_with("-service");
    card.activities
        .iter()
        .filter(|a| q.includes(a.starts_at))
        .filter(|a| !(service && a.minimum > 0 && a.committed >= a.minimum))
        .filter_map(|a| {
            let (place, label) = match &a.place {
                ActivityPlace::Own(p) => (p, &a.label),
                ActivityPlace::Parent => (card.place.as_ref()?, &card.location_label),
                ActivityPlace::Elsewhere => return None,
            };
            let distance = sp_place::distance_km(q.center, (place.latitude, place.longitude));
            (distance <= q.radius_km).then(|| Marker {
                id: a.id.clone(),
                entity_type: "activity",
                activity_source: Some(source),
                project_mode: (source == "project").then(|| card.mode.clone()),
                title: a.title.clone(),
                parent_id: Some(card.id.clone()),
                parent_title: Some(card.title.clone()),
                subtitle: Some(a.title.clone()),
                href: format!("/{}/{}?activity={}", path_of(kind), card.id, a.id),
                latitude: place.latitude,
                longitude: place.longitude,
                precision: place.precision.clone(),
                display_label: label.clone(),
                distance_km: round_km(distance),
                scheduled_at: Some(a.starts_at),
                ends_at: Some(a.ends_at),
                signup_count: None,
                slots_needed: None,
                committed_count: Some(a.committed),
                minimum_participants: Some(a.minimum),
            })
        })
        .collect()
}

/// web's markers: pins within the radius, timed ones only while their time is wanted, soonest
/// first, then untimed ones.
pub fn markers(cards: &[ProjectCard], q: &MarkerQuery) -> Vec<Marker> {
    let mut out: Vec<Marker> = cards
        .iter()
        .filter(|c| c.moderation.as_deref() != Some(super::index::REMOVED))
        .flat_map(|c| {
            let Some(kind) = kind_of(c, q.filter) else {
                return Vec::new();
            };
            let mut found = activity_markers(c, kind, q);
            found.extend(card_marker(c, kind, q));
            found
        })
        .collect();
    out.sort_by_key(|m| m.scheduled_at.unwrap_or(i64::MAX));
    out.truncate(q.limit);
    out
}

#[cfg(test)]
#[path = "markers_tests.rs"]
mod tests;
