//! Provider answers turned into web's `LocationResult` (web-backend
//! `app/services/locations/geocoding.py::_normalize_nominatim_result` and `ip_hint.py`), and
//! web's checks on the query and coordinates. Pure, so it is unit-tested without a network.

use serde::Serialize;
use serde_json::Value;
use sp_place::{MAX_AREA, MAX_LABEL};

/// web's `MAX_SEARCH_QUERY_LENGTH`.
pub const MAX_QUERY: usize = 200;
/// web's `MAX_SEARCH_RESULTS`.
pub const MAX_RESULTS: usize = 10;

/// web's `LocationResult`, field for field (snake case, as web's driver reads it).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Location {
    pub provider_place_id: Option<String>,
    pub display_label: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub precision: &'static str,
    pub is_online: bool,
}

/// web's error codes (its `detail`), sent as the gateway's `error`.
pub fn query(q: &str) -> Result<String, &'static str> {
    let cleaned = q.trim();
    if cleaned.is_empty() {
        return Err("geocoding_query_required");
    }
    if cleaned.chars().count() > MAX_QUERY {
        return Err("geocoding_query_too_long");
    }
    Ok(cleaned.to_string())
}

pub fn coordinates(lat: f64, lon: f64) -> Result<(f64, f64), &'static str> {
    if !lat.is_finite() || !lon.is_finite() {
        return Err("invalid_location_coordinates");
    }
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return Err("location_coordinates_out_of_range");
    }
    Ok((lat, lon))
}

/// web's `viewbox` parameter: four comma-separated numbers, else ignored.
#[must_use]
pub fn viewbox(raw: &str) -> Option<[f64; 4]> {
    let parts: Vec<f64> = raw
        .split(',')
        .map(|p| p.trim().parse::<f64>().ok())
        .collect::<Option<_>>()?;
    parts.try_into().ok()
}

/// Every result of a Nominatim answer (an object for `/reverse`, a list for `/search`).
pub fn nominatim(payload: &Value) -> Vec<Location> {
    let items = match payload {
        Value::Array(items) => items.as_slice(),
        Value::Object(_) => std::slice::from_ref(payload),
        _ => &[],
    };
    items
        .iter()
        .filter_map(nominatim_one)
        .take(MAX_RESULTS)
        .collect()
}

fn nominatim_one(item: &Value) -> Option<Location> {
    let lat = number(item.get("lat")?)?;
    let lon = number(item.get("lon")?)?;
    let address = item.get("address").filter(|a| a.is_object());
    let field = |k: &str| address.and_then(|a| text(a.get(k)));
    let region = ["state", "region", "county", "city"]
        .into_iter()
        .find_map(field);
    let label = text(item.get("display_name"))?;
    let provider_place_id = match (
        item.get("place_id"),
        item.get("osm_type"),
        item.get("osm_id"),
    ) {
        (Some(id), _, _) if !id.is_null() => Some(format!("nominatim:{}", plain(id))),
        (_, Some(kind), Some(id)) if !id.is_null() && text(Some(kind)).is_some() => {
            Some(format!("osm:{}:{}", plain(kind), plain(id)))
        }
        _ => None,
    };
    Some(Location {
        provider_place_id,
        display_label: cut(&label, MAX_LABEL),
        latitude: Some(round6(lat)),
        longitude: Some(round6(lon)),
        region: region.map(|r| cut(&r, MAX_AREA)),
        country: field("country").map(|c| cut(&c, MAX_AREA)),
        precision: "approximate",
        is_online: false,
    })
}

/// ip-api.com's answer (`fields=status,message,country,regionName,city,lat,lon`).
pub fn ip_api(payload: &Value) -> Result<Location, &'static str> {
    if payload.get("status").and_then(Value::as_str) != Some("success") {
        return Err("ip_location_unavailable");
    }
    let lat = payload.get("lat").and_then(number);
    let lon = payload.get("lon").and_then(number);
    let (Some(lat), Some(lon)) = (lat, lon) else {
        return Err("ip_location_provider_invalid_response");
    };
    let (lat, lon) = coordinates(lat, lon)?;
    let city = text(payload.get("city"));
    let region = text(payload.get("regionName"));
    let country = text(payload.get("country"));
    let parts: Vec<&str> = [&city, &region, &country]
        .into_iter()
        .filter_map(|p| p.as_deref())
        .collect();
    let label = if parts.is_empty() {
        format!("{lat:.2}, {lon:.2}")
    } else {
        parts.join(", ")
    };
    Ok(Location {
        provider_place_id: None,
        display_label: cut(&label, MAX_LABEL),
        latitude: Some(round6(lat)),
        longitude: Some(round6(lon)),
        region: region.map(|r| cut(&r, MAX_AREA)),
        country: country.map(|c| cut(&c, MAX_AREA)),
        precision: "approximate",
        is_online: false,
    })
}

/// Nominatim sends coordinates as strings, ip-api as numbers.
fn number(v: &Value) -> Option<f64> {
    match v {
        Value::String(s) => s.trim().parse().ok(),
        other => other.as_f64(),
    }
    .filter(|n: &f64| n.is_finite())
}

/// A non-empty string or number, trimmed (web's `str(x).strip()` on a truthy value).
fn text(v: Option<&Value>) -> Option<String> {
    let s = match v? {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    (!s.is_empty()).then_some(s)
}

fn plain(v: &Value) -> String {
    v.as_str().map_or_else(|| v.to_string(), str::to_string)
}

fn cut(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

fn round6(v: f64) -> f64 {
    (v * 1e6).round() / 1e6
}
