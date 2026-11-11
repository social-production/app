//! web's `geocoding_*` settings, read from the environment (docs/DEPLOYMENT.md).

use std::time::Duration;

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
        Self::from_env_for("p2p seed node")
    }

    /// The same, with `who` in the default User-Agent (Nominatim asks each app to say who
    /// it is): "p2p seed node" on the gateway, "desktop app" on a device.
    #[must_use]
    pub fn from_env_for(who: &str) -> Self {
        let var = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
        let num = |k: &str, d: f64| var(k, "").parse::<f64>().unwrap_or(d).max(0.0);
        Self {
            provider_url: var(
                "GEOCODING_PROVIDER_URL",
                "https://nominatim.openstreetmap.org",
            ),
            api_key: var("GEOCODING_PROVIDER_API_KEY", ""),
            user_agent: std::env::var("GEOCODING_USER_AGENT").unwrap_or_else(|_| {
                format!("SocialProduction/0.1 ({who}; github.com/social-production/app)")
            }),
            cache_ttl: Duration::from_secs_f64(num("GEOCODING_CACHE_TTL_SECONDS", 86_400.0)),
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            per_ip: num("GEOCODING_RATE_LIMIT", 30.0) as u32,
            window: Duration::from_secs_f64(num("GEOCODING_RATE_LIMIT_WINDOW_SECONDS", 60.0)),
            timeout: Duration::from_secs_f64(num("GEOCODING_TIMEOUT_SECONDS", 8.0)),
            ip_hint_url: var("IP_HINT_URL", "http://ip-api.com/json/"),
        }
    }
}

impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("provider_url", &self.provider_url)
            .field("user_agent", &self.user_agent)
            .finish_non_exhaustive()
    }
}
