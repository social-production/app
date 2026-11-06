//! Places, as web-backend's `locations` table keeps them (`app/services/locations/model.py`):
//! a label people read, coordinates unless the place is online, an optional region and
//! country, the geocoder's id, and a precision. Approximate places keep two decimals of each
//! coordinate (about a kilometre), exact ones six. Plan: `docs/plans/M6-places.md`.

/// web's `display_label` limit.
pub const MAX_LABEL: usize = 240;
/// web's `region` and `country` limits.
pub const MAX_AREA: usize = 120;
/// web's `provider_place_id` limit.
pub const MAX_PROVIDER_ID: usize = 160;

/// web's `LocationPrecision`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precision {
    Exact,
    Approximate,
}

impl Precision {
    /// web's names (`exact`, `approximate`).
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "exact" => Ok(Self::Exact),
            "approximate" => Ok(Self::Approximate),
            _ => Err(format!("unknown place precision {s:?}")),
        }
    }

    /// Decimals kept on each coordinate (web's `APPROXIMATE_COORD_DECIMALS`, `EXACT_…`).
    #[must_use]
    pub fn decimals(self) -> i32 {
        match self {
            Self::Exact => 6,
            Self::Approximate => 2,
        }
    }
}

/// A coordinate rounded to `precision` (what writers store and validators expect).
#[must_use]
pub fn round(value: f64, precision: Precision) -> f64 {
    let scale = 10f64.powi(precision.decimals());
    (value * scale).round() / scale
}

/// A place as its entry stores it, borrowed for checking.
#[derive(Debug)]
pub struct PlaceRef<'a> {
    pub label: &'a str,
    pub coordinates: Option<(f64, f64)>,
    pub region: Option<&'a str>,
    pub country: Option<&'a str>,
    pub provider_id: Option<&'a str>,
    pub precision: &'a str,
    pub online: bool,
}

fn optional(field: &str, value: Option<&str>, max: usize) -> Result<(), String> {
    match value {
        Some(v) if v.trim().is_empty() => Err(format!("an empty place {field} is left out")),
        Some(v) if v.chars().count() > max => Err(format!("place {field} over {max} characters")),
        _ => Ok(()),
    }
}

/// web's `validate_location_input`: a label, coordinates in range and rounded to the
/// precision for a physical place, none for an online one.
pub fn check(p: &PlaceRef) -> Result<(), String> {
    if p.label.trim().is_empty() {
        return Err("a place needs a label".into());
    }
    if p.label.chars().count() > MAX_LABEL {
        return Err(format!("place label over {MAX_LABEL} characters"));
    }
    optional("region", p.region, MAX_AREA)?;
    optional("country", p.country, MAX_AREA)?;
    optional("provider id", p.provider_id, MAX_PROVIDER_ID)?;
    let precision = Precision::parse(p.precision)?;
    match (p.online, p.coordinates) {
        (true, Some(_)) => Err("an online place has no coordinates".into()),
        (true, None) => Ok(()),
        (false, None) => Err("a physical place needs coordinates".into()),
        (false, Some((lat, lon))) => {
            if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
                return Err("place coordinates out of range".into());
            }
            let off = |v: f64| (round(v, precision) - v).abs() > 1e-9;
            if off(lat) || off(lon) {
                return Err("place coordinates carry more decimals than their precision".into());
            }
            Ok(())
        }
    }
}

/// web-backend's `haversine_km` (Region feed and map distances).
#[must_use]
pub fn distance_km(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (lat1, lon1) = (a.0.to_radians(), a.1.to_radians());
    let (lat2, lon2) = (b.0.to_radians(), b.1.to_radians());
    let h = ((lat2 - lat1) / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * ((lon2 - lon1) / 2.0).sin().powi(2);
    2.0 * 6371.0088 * h.sqrt().asin()
}

#[cfg(test)]
mod tests;
