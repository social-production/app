//! Places on feed cards and web's map markers (M6 phase 4a, ADR 0082): web-backend's
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

/// web's `/feeds/map-markers` parameters this phase serves (`window` and dates come with 4b).
pub struct MarkerQuery<'a> {
    pub center: (f64, f64),
    pub radius_km: f64,
    /// `all`, `projects`, `events` or `help_requests`.
    pub filter: &'a str,
    pub upcoming_only: bool,
    pub limit: usize,
    /// Holochain microseconds.
    pub now: i64,
}

/// web's `MapMarkerOut` (snake case, as web's frontend reads it).
#[derive(Debug, Serialize)]
pub struct Marker {
    pub id: String,
    pub entity_type: &'static str,
    pub project_mode: Option<String>,
    pub title: String,
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
}

/// What a card is on the map, or `None` when web shows no pin for it.
fn kind_of(card: &ProjectCard, filter: &str) -> Option<&'static str> {
    let kind = match card.mode.as_str() {
        "event" => "event",
        "help-request" => "help_request",
        // web's `_project_pin_eligible`; a personal service's requests are on by default in
        // the app (ADR 0077), and the index doesn't read the setting.
        "productive" | "collective-service" | "personal-service" => "project",
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

/// web's `_project_pin_subtitle`, without open spots (activities come with 4b).
fn subtitle(card: &ProjectCard) -> String {
    match card.mode.as_str() {
        "personal-service" => "Accepting requests".into(),
        _ if card.phase.trim().is_empty() => "Proposal".into(),
        _ => card.phase.trim().to_owned(),
    }
}

fn marker(card: &ProjectCard, kind: &'static str, place: &CardPlace, distance: f64) -> Marker {
    let (path, project_mode) = match kind {
        "event" => ("events", None),
        "help_request" => ("help-requests", None),
        _ => ("projects", Some(card.mode.clone())),
    };
    let help = card.help.as_ref();
    Marker {
        id: card.id.clone(),
        entity_type: kind,
        project_mode,
        title: card.title.clone(),
        subtitle: (kind == "project").then(|| subtitle(card)),
        href: format!("/{path}/{}", card.id),
        latitude: place.latitude,
        longitude: place.longitude,
        precision: place.precision.clone(),
        display_label: card.location_label.clone(),
        distance_km: (distance * 100.0).round() / 100.0,
        scheduled_at: match kind {
            "event" => card.scheduled_at,
            "help_request" => help.and_then(|h| h.needed_at),
            _ => None,
        },
        ends_at: help.and_then(|h| h.ends_at),
        signup_count: help.map(|h| h.signup_count),
        slots_needed: help.map(|h| h.slots_needed),
    }
}

/// web's markers: pinned cards within the radius, timed ones only once scheduled (and, with
/// `upcoming_only`, not yet started), soonest first, then untimed ones.
pub fn markers(cards: &[ProjectCard], q: &MarkerQuery) -> Vec<Marker> {
    let mut out: Vec<Marker> = cards
        .iter()
        .filter(|c| c.moderation.as_deref() != Some(super::index::REMOVED))
        .filter_map(|c| {
            let kind = kind_of(c, q.filter)?;
            let place = c.place.as_ref()?;
            let distance = sp_place::distance_km(q.center, (place.latitude, place.longitude));
            if distance > q.radius_km {
                return None;
            }
            let m = marker(c, kind, place, distance);
            let timed = kind != "project";
            if timed && m.scheduled_at.is_none() {
                return None;
            }
            if timed && q.upcoming_only && m.scheduled_at.is_some_and(|t| t < q.now) {
                return None;
            }
            Some(m)
        })
        .collect();
    out.sort_by_key(|m| m.scheduled_at.unwrap_or(i64::MAX));
    out.truncate(q.limit);
    out
}

#[cfg(test)]
#[path = "markers_tests.rs"]
mod tests;
