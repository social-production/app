//! Place lookups from this device (SEEDLESS step 5, ADR 0093): the gateway's
//! `/locations/search`, `/reverse` and `/ip-hint` (ADR 0079), run here with the same code
//! (`services/geo`), so the place picker works with no gateway. Nominatim sees this device's
//! address and the desktop app's User-Agent; the limits (30 lookups a minute, 10 IP hints)
//! keep it within Nominatim's one-a-second policy.

use std::collections::HashMap;
use std::sync::OnceLock;

use geo::{Geo, GeoError, Location, Settings};
use serde::Serialize;

#[derive(Serialize)]
pub struct Items {
    items: Vec<Location>,
}

/// A refusal as the gateway sends it: its status and web's error code.
#[derive(Debug, Serialize)]
pub struct Refusal {
    status: u16,
    error: String,
}

impl From<GeoError> for Refusal {
    fn from(GeoError(status, code): GeoError) -> Self {
        Self {
            status,
            error: code.into(),
        }
    }
}

/// The limits count every lookup on this device under one key.
const DEVICE: &str = "device";

fn geo() -> Result<&'static Geo, Refusal> {
    static GEO: OnceLock<Option<Geo>> = OnceLock::new();
    GEO.get_or_init(|| {
        Geo::new(Settings::from_env_for("desktop app"))
            .map_err(|e| log::error!("geocoder: {e}"))
            .ok()
    })
    .as_ref()
    .ok_or(Refusal {
        status: geo::status::SERVICE_UNAVAILABLE,
        error: "geocoding_provider_unavailable".into(),
    })
}

fn invalid(code: &str) -> Refusal {
    Refusal {
        status: geo::status::UNPROCESSABLE_ENTITY,
        error: code.into(),
    }
}

/// `route` is `search`, `reverse` or `ip-hint`; `query` holds the gateway route's parameters.
#[tauri::command]
pub async fn locations(route: String, query: HashMap<String, String>) -> Result<Items, Refusal> {
    let geo = geo()?;
    let arg = |k: &str| query.get(k).map(String::as_str);
    let items = match route.as_str() {
        "search" => {
            let q = arg("q").unwrap_or_default();
            let limit = arg("limit").map_or(Ok(5), str::parse::<usize>);
            let limit = limit.map_err(|_| invalid("invalid_location_search"))?;
            let (codes, viewbox) = (arg("country_codes"), arg("viewbox"));
            geo::check_search(q, limit, codes, viewbox)?;
            geo.check_limit(DEVICE)?;
            let viewbox = viewbox.and_then(geo::shape::viewbox);
            geo.search(q, limit, codes, viewbox).await?
        }
        "reverse" => {
            let coordinate = |k: &str| arg(k).and_then(|v| v.parse::<f64>().ok());
            let (Some(lat), Some(lon)) = (coordinate("lat"), coordinate("lon")) else {
                return Err(invalid("invalid_location_coordinates"));
            };
            geo.check_limit(DEVICE)?;
            geo.reverse(lat, lon).await?
        }
        "ip-hint" => vec![geo.own_ip_hint().await?],
        _ => return Err(invalid("unknown_location_route")),
    };
    Ok(Items { items })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(route: &str, query: &[(&str, &str)]) -> Result<Items, Refusal> {
        let query = query
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        tauri::async_runtime::block_on(locations(route.into(), query))
    }

    fn code(answer: Result<Items, Refusal>) -> (u16, String) {
        let refusal = answer.err().expect("a refusal");
        (refusal.status, refusal.error)
    }

    /// The checks run before any network call, with the gateway's codes.
    #[test]
    fn refuses_bad_lookups_like_the_gateway() {
        let invalid = |c: &str| (422, c.to_string());
        assert_eq!(code(ask("search", &[])), invalid("invalid_location_search"));
        let many = [("q", "Fitzroy"), ("limit", "11")];
        assert_eq!(
            code(ask("search", &many)),
            invalid("invalid_location_search")
        );
        let words = [("q", "Fitzroy"), ("limit", "five")];
        assert_eq!(
            code(ask("search", &words)),
            invalid("invalid_location_search")
        );
        let half = [("lat", "1")];
        assert_eq!(
            code(ask("reverse", &half)),
            invalid("invalid_location_coordinates")
        );
        let far = [("lat", "91"), ("lon", "0")];
        assert_eq!(
            code(ask("reverse", &far)),
            invalid("location_coordinates_out_of_range")
        );
        assert_eq!(code(ask("map", &[])), invalid("unknown_location_route"));
    }
}
