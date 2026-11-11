//! Geocoding for the place picker (M6 phase 2, ADR 0079): web-backend's `/locations/search`,
//! `/reverse` and `/ip-hint` (`app/services/locations/geocoding.py`, `ip_hint.py`) against
//! Nominatim and ip-api.com, with web's cache, per-IP limits and error codes. Places are
//! never stored here: what someone picks goes onto their own records (ADR 0078). The gateway
//! serves it over HTTP; the desktop app calls it in-process, from the device (ADR 0093).

mod memo;
mod settings;
pub mod shape;
#[cfg(test)]
mod tests;

use std::time::Duration;

use memo::{Cache, WindowLimit};
pub use settings::Settings;
pub use shape::Location;

/// Calls this node makes to the geocoder per minute, whoever asks: Nominatim's usage policy
/// allows about one a second from one server. web has no such cap; cached answers skip it.
const PROVIDER_PER_MINUTE: u32 = 60;
/// web's `enforce_ip_hint_rate_limit`: 10 hints per IP per window.
const HINTS_PER_WINDOW: u32 = 10;

/// HTTP statuses web answers with (the gateway sends them; the desktop passes them to the UI).
pub mod status {
    pub const BAD_GATEWAY: u16 = 502;
    pub const SERVICE_UNAVAILABLE: u16 = 503;
    pub const TOO_MANY_REQUESTS: u16 = 429;
    pub const UNPROCESSABLE_ENTITY: u16 = 422;
}
use status::{BAD_GATEWAY, SERVICE_UNAVAILABLE, TOO_MANY_REQUESTS, UNPROCESSABLE_ENTITY};

/// A refusal in web's words: its HTTP status and its `detail` code.
#[derive(Debug, PartialEq, Eq)]
pub struct GeoError(pub u16, pub &'static str);

fn unprocessable(code: &'static str) -> GeoError {
    GeoError(UNPROCESSABLE_ENTITY, code)
}

fn hint_rate_limited() -> GeoError {
    GeoError(TOO_MANY_REQUESTS, "ip_location_rate_limit_exceeded")
}

/// web's FastAPI checks on a search: q 1–200 characters, limit 1–10, short extras.
pub fn check_search(
    q: &str,
    limit: usize,
    country_codes: Option<&str>,
    viewbox: Option<&str>,
) -> Result<(), GeoError> {
    let too_long = |v: Option<&str>, max: usize| v.is_some_and(|s| s.len() > max);
    if q.is_empty()
        || q.chars().count() > shape::MAX_QUERY
        || !(1..=shape::MAX_RESULTS).contains(&limit)
        || too_long(country_codes, 40)
        || too_long(viewbox, 80)
    {
        return Err(unprocessable("invalid_location_search"));
    }
    Ok(())
}

pub struct Geo {
    settings: Settings,
    http: reqwest::Client,
    answers: Cache<Vec<Location>>,
    hints: Cache<Location>,
    per_ip: WindowLimit,
    hint_limit: WindowLimit,
    provider: WindowLimit,
}

impl std::fmt::Debug for Geo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Geo")
            .field("provider_url", &self.settings.provider_url)
            .finish_non_exhaustive()
    }
}

impl Geo {
    pub fn new(settings: Settings) -> anyhow::Result<Self> {
        Self::with_client(settings, reqwest::Client::builder())
    }

    /// Tests pass a builder that skips the environment's proxy.
    fn with_client(settings: Settings, builder: reqwest::ClientBuilder) -> anyhow::Result<Self> {
        let http = builder
            .timeout(settings.timeout)
            .user_agent(settings.user_agent.clone())
            .build()?;
        Ok(Self {
            answers: Cache::new(settings.cache_ttl.max(Duration::from_mins(1))),
            hints: Cache::new(settings.cache_ttl.max(Duration::from_mins(5))),
            per_ip: WindowLimit::new(settings.per_ip, settings.window),
            hint_limit: WindowLimit::new(HINTS_PER_WINDOW, settings.window),
            provider: WindowLimit::new(PROVIDER_PER_MINUTE, Duration::from_mins(1)),
            http,
            settings,
        })
    }

    /// web's `enforce_geocoding_rate_limit`.
    pub fn check_limit(&self, ip: &str) -> Result<(), GeoError> {
        if self.per_ip.allow(ip) {
            Ok(())
        } else {
            Err(GeoError(TOO_MANY_REQUESTS, "geocoding_rate_limit_exceeded"))
        }
    }

    /// web's `search_places`.
    pub async fn search(
        &self,
        q: &str,
        limit: usize,
        country_codes: Option<&str>,
        viewbox: Option<[f64; 4]>,
    ) -> Result<Vec<Location>, GeoError> {
        let cleaned = shape::query(q).map_err(unprocessable)?;
        let limit = limit.clamp(1, shape::MAX_RESULTS);
        let codes = country_codes.map(str::trim).filter(|c| !c.is_empty());
        let codes = codes.map(str::to_lowercase);
        let rounded = viewbox.map(|v| v.map(|x| (x * 1e4).round() / 1e4));
        let key = format!(
            "search|{}|{limit}|{}|{rounded:?}",
            cleaned.to_lowercase(),
            codes.clone().unwrap_or_default()
        );
        if let Some(hit) = self.answers.get(&key) {
            return Ok(hit.into_iter().take(limit).collect());
        }
        let mut params = vec![
            ("q", cleaned),
            ("format", "json".into()),
            ("addressdetails", "1".into()),
            ("limit", limit.to_string()),
        ];
        if let Some(codes) = codes {
            params.push(("countrycodes", codes));
        }
        if let Some([min_lon, max_lat, max_lon, min_lat]) = viewbox {
            params.push((
                "viewbox",
                format!("{min_lon},{max_lat},{max_lon},{min_lat}"),
            ));
            params.push(("bounded", "0".into()));
        }
        let found = self.provider_get("/search", params).await?;
        self.answers.put(key, found.clone());
        Ok(found)
    }

    /// web's `reverse_geocode`.
    pub async fn reverse(&self, lat: f64, lon: f64) -> Result<Vec<Location>, GeoError> {
        let (lat, lon) = shape::coordinates(lat, lon).map_err(unprocessable)?;
        let key = format!("reverse|{lat:.6}|{lon:.6}");
        if let Some(hit) = self.answers.get(&key) {
            return Ok(hit);
        }
        let params = vec![
            ("lat", format!("{lat:.6}")),
            ("lon", format!("{lon:.6}")),
            ("format", "json".into()),
            ("addressdetails", "1".into()),
        ];
        let found = self.provider_get("/reverse", params).await?;
        self.answers.put(key, found.clone());
        Ok(found)
    }

    /// web's `ip_location_hint` for a client of the gateway: only on the person's explicit
    /// ask (Settings' Regional).
    pub async fn ip_hint(&self, ip: &str) -> Result<Location, GeoError> {
        if ip
            .parse::<std::net::IpAddr>()
            .map_or(true, |a| a.is_loopback())
        {
            if !self.hint_limit.allow(ip) {
                return Err(hint_rate_limited());
            }
            return Err(unprocessable("ip_location_unavailable"));
        }
        self.hint_for(ip, ip).await
    }

    /// The same for this device's own address (the desktop app, ADR 0093): ip-api.com reads
    /// the address the request comes from.
    pub async fn own_ip_hint(&self) -> Result<Location, GeoError> {
        self.hint_for("device", "").await
    }

    async fn hint_for(&self, key: &str, ip: &str) -> Result<Location, GeoError> {
        if !self.hint_limit.allow(key) {
            return Err(hint_rate_limited());
        }
        if let Some(hit) = self.hints.get(key) {
            return Ok(hit);
        }
        let url = format!(
            "{}{ip}?fields=status,message,country,regionName,city,lat,lon",
            self.settings.ip_hint_url
        );
        let unavailable = || GeoError(SERVICE_UNAVAILABLE, "ip_location_provider_unavailable");
        let response = self.http.get(url).send().await.map_err(|_| unavailable())?;
        if response.status().as_u16() >= 400 {
            return Err(unavailable());
        }
        let payload: serde_json::Value = response
            .json()
            .await
            .map_err(|_| GeoError(BAD_GATEWAY, "ip_location_provider_invalid_response"))?;
        let hint = shape::ip_api(&payload).map_err(unprocessable)?;
        self.hints.put(key.to_string(), hint.clone());
        Ok(hint)
    }

    /// web's `_provider_get`, with its status mapping.
    async fn provider_get(
        &self,
        path: &str,
        mut params: Vec<(&'static str, String)>,
    ) -> Result<Vec<Location>, GeoError> {
        let unavailable = GeoError(SERVICE_UNAVAILABLE, "geocoding_provider_unavailable");
        if !self.provider.allow("provider") {
            return Err(GeoError(
                TOO_MANY_REQUESTS,
                "geocoding_provider_rate_limited",
            ));
        }
        if !self.settings.api_key.trim().is_empty() {
            params.push(("key", self.settings.api_key.trim().to_string()));
        }
        let url = format!("{}{path}", self.settings.provider_url.trim_end_matches('/'));
        let sent = self
            .http
            .get(url)
            .query(&params)
            .header("Accept", "application/json")
            .send()
            .await;
        let response = match sent {
            Ok(r) => r,
            Err(e) if e.is_timeout() => {
                return Err(GeoError(SERVICE_UNAVAILABLE, "geocoding_provider_timeout"))
            }
            Err(_) => return Err(unavailable),
        };
        match response.status().as_u16() {
            429 => {
                return Err(GeoError(
                    TOO_MANY_REQUESTS,
                    "geocoding_provider_rate_limited",
                ))
            }
            500.. => return Err(unavailable),
            400.. => return Err(GeoError(BAD_GATEWAY, "geocoding_provider_error")),
            _ => {}
        }
        let payload: serde_json::Value = response
            .json()
            .await
            .map_err(|_| GeoError(BAD_GATEWAY, "geocoding_provider_invalid_response"))?;
        Ok(shape::nominatim(&payload))
    }
}
