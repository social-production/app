//! Geocoding for the place picker (M6 phase 2, ADR 0079): web-backend's `/locations/search`,
//! `/reverse` and `/ip-hint` (`app/services/locations/geocoding.py`, `ip_hint.py`) proxied to
//! Nominatim and ip-api.com, with web's cache, per-IP limits and error codes. Places are
//! never stored here: what someone picks goes onto their own records (ADR 0078).

mod memo;
pub mod shape;
#[cfg(test)]
mod tests;

use std::time::Duration;

use axum::http::StatusCode;
use memo::{Cache, WindowLimit};
pub use shape::Location;

/// Calls this node makes to the geocoder per minute, whoever asks: Nominatim's usage policy
/// allows about one a second from one server. web has no such cap; cached answers skip it.
const PROVIDER_PER_MINUTE: u32 = 60;
/// web's `enforce_ip_hint_rate_limit`: 10 hints per IP per window.
const HINTS_PER_WINDOW: u32 = 10;

/// web's `geocoding_*` settings, from the same names upper-cased (docs/DEPLOYMENT.md).
#[derive(Clone)]
pub struct Settings {
    pub provider_url: String,
    pub api_key: String,
    pub user_agent: String,
    pub cache_ttl: Duration,
    pub per_ip: u32,
    pub window: Duration,
    pub timeout: Duration,
    /// ip-api.com's lookup; the client's IP is appended.
    pub ip_hint_url: String,
}

impl Settings {
    #[must_use]
    pub fn from_env() -> Self {
        let var = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
        let num = |k: &str, d: f64| var(k, "").parse::<f64>().unwrap_or(d).max(0.0);
        Self {
            provider_url: var(
                "GEOCODING_PROVIDER_URL",
                "https://nominatim.openstreetmap.org",
            ),
            api_key: var("GEOCODING_PROVIDER_API_KEY", ""),
            user_agent: var(
                "GEOCODING_USER_AGENT",
                "SocialProduction/0.1 (p2p seed node; github.com/social-production/app)",
            ),
            cache_ttl: Duration::from_secs_f64(num("GEOCODING_CACHE_TTL_SECONDS", 86_400.0)),
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            per_ip: num("GEOCODING_RATE_LIMIT", 30.0) as u32,
            window: Duration::from_secs_f64(num("GEOCODING_RATE_LIMIT_WINDOW_SECONDS", 60.0)),
            timeout: Duration::from_secs_f64(num("GEOCODING_TIMEOUT_SECONDS", 8.0)),
            ip_hint_url: var("IP_HINT_URL", "http://ip-api.com/json/"),
        }
    }
}

/// A refusal in web's words: its status and its `detail` code.
#[derive(Debug, PartialEq, Eq)]
pub struct GeoError(pub StatusCode, pub &'static str);

fn unprocessable(code: &'static str) -> GeoError {
    GeoError(StatusCode::UNPROCESSABLE_ENTITY, code)
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
            Err(GeoError(
                StatusCode::TOO_MANY_REQUESTS,
                "geocoding_rate_limit_exceeded",
            ))
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

    /// web's `ip_location_hint`: only on the person's explicit ask (Settings' Regional).
    pub async fn ip_hint(&self, ip: &str) -> Result<Location, GeoError> {
        if !self.hint_limit.allow(ip) {
            return Err(GeoError(
                StatusCode::TOO_MANY_REQUESTS,
                "ip_location_rate_limit_exceeded",
            ));
        }
        if ip
            .parse::<std::net::IpAddr>()
            .map_or(true, |a| a.is_loopback())
        {
            return Err(unprocessable("ip_location_unavailable"));
        }
        if let Some(hit) = self.hints.get(ip) {
            return Ok(hit);
        }
        let url = format!(
            "{}{ip}?fields=status,message,country,regionName,city,lat,lon",
            self.settings.ip_hint_url
        );
        let unavailable = GeoError(
            StatusCode::SERVICE_UNAVAILABLE,
            "ip_location_provider_unavailable",
        );
        let response = self.http.get(url).send().await.map_err(|_| unavailable)?;
        if response.status().as_u16() >= 400 {
            return Err(GeoError(
                StatusCode::SERVICE_UNAVAILABLE,
                "ip_location_provider_unavailable",
            ));
        }
        let payload: serde_json::Value = response.json().await.map_err(|_| {
            GeoError(
                StatusCode::BAD_GATEWAY,
                "ip_location_provider_invalid_response",
            )
        })?;
        let hint = shape::ip_api(&payload).map_err(unprocessable)?;
        self.hints.put(ip.to_string(), hint.clone());
        Ok(hint)
    }

    /// web's `_provider_get`, with its status mapping.
    async fn provider_get(
        &self,
        path: &str,
        mut params: Vec<(&'static str, String)>,
    ) -> Result<Vec<Location>, GeoError> {
        let unavailable = GeoError(
            StatusCode::SERVICE_UNAVAILABLE,
            "geocoding_provider_unavailable",
        );
        if !self.provider.allow("provider") {
            return Err(GeoError(
                StatusCode::TOO_MANY_REQUESTS,
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
                return Err(GeoError(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "geocoding_provider_timeout",
                ))
            }
            Err(_) => return Err(unavailable),
        };
        match response.status().as_u16() {
            429 => {
                return Err(GeoError(
                    StatusCode::TOO_MANY_REQUESTS,
                    "geocoding_provider_rate_limited",
                ))
            }
            500.. => return Err(unavailable),
            400.. => {
                return Err(GeoError(
                    StatusCode::BAD_GATEWAY,
                    "geocoding_provider_error",
                ))
            }
            _ => {}
        }
        let payload: serde_json::Value = response.json().await.map_err(|_| {
            GeoError(
                StatusCode::BAD_GATEWAY,
                "geocoding_provider_invalid_response",
            )
        })?;
        Ok(shape::nominatim(&payload))
    }
}
