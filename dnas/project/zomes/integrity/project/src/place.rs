//! Places (M6, ADR 0078): web-backend's `locations` row, stored next to the label a record
//! already has. The label stays what cards show; a place adds coordinates for the map and the
//! Region feed. Rules in `sp_place`.

use hdi::prelude::*;

/// web's location: `display_label`, coordinates unless online, region, country, the
/// geocoder's id and a precision (`exact` or `approximate`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Place {
    pub label: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub provider_id: Option<String>,
    pub precision: String,
    pub online: bool,
}

/// The place is valid, and its label is the record's own `label` (so cards and the map agree).
pub fn check(place: &Place, label: &str) -> Result<(), String> {
    let coordinates = match (place.latitude, place.longitude) {
        (Some(lat), Some(lon)) => Some((lat, lon)),
        (None, None) => None,
        _ => return Err("a place has both coordinates or neither".into()),
    };
    sp_place::check(&sp_place::PlaceRef {
        label: &place.label,
        coordinates,
        region: place.region.as_deref(),
        country: place.country.as_deref(),
        provider_id: place.provider_id.as_deref(),
        precision: &place.precision,
        online: place.online,
    })?;
    if place.label != label {
        return Err("a place's label must match the record's location label".into());
    }
    Ok(())
}

/// `check` for an optional place.
pub fn check_optional(place: Option<&Place>, label: Option<&str>) -> Result<(), String> {
    match (place, label) {
        (None, _) => Ok(()),
        (Some(p), Some(l)) => check(p, l),
        (Some(_), None) => Err("a place needs its location label".into()),
    }
}
