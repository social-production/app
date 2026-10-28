use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

/// Gateway settings, read once from the environment at startup (docs/DEPLOYMENT.md).
/// The keystore passphrase is read separately so it never lands in a `Debug` print.
#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    /// Conductor data, keystore and the gateway database (a Railway volume in production).
    pub data_dir: PathBuf,
    /// Conductor app-websocket port, localhost only; `/ws` forwards to it.
    pub app_ws_port: u16,
    pub happ_path: PathBuf,
    pub bootstrap_url: String,
    pub relay_url: String,
    /// `WebAuthn` relying party: the UI's domain and origin (passkeys are bound to it).
    pub rp_id: String,
    pub rp_origin: String,
    /// Signup proof-of-work difficulty in leading zero bits (20 ≈ a few seconds).
    pub pow_bits: u32,
    pub signups_per_ip_per_day: u32,
    /// Readers per database (`SQLite`). Every project (each its own DNA) opens two databases, and
    /// `sqlx` runs each connection on its own thread with its own page cache, so Holochain's
    /// default (2 × CPUs, at least 8) costs memory per project (ADR 0062).
    pub db_max_readers: u16,
    /// Exit after removing apps of older hApp versions, so the platform restarts a process
    /// without their compiled zomes (ADR 0064). Railway's `ON_FAILURE` policy restarts it.
    pub restart_after_upgrade: bool,
}

impl Config {
    #[must_use]
    pub fn from_env() -> Self {
        let bootstrap_url = env_or("BOOTSTRAP_URL", "https://dev-test-bootstrap2.holochain.org");
        Self {
            bind_addr: SocketAddr::from((Ipv4Addr::UNSPECIFIED, env_num("PORT", 8080))),
            data_dir: PathBuf::from(env_or("DATA_DIR", "./.gateway-data")),
            app_ws_port: env_num("APP_WS_PORT", 8888),
            happ_path: PathBuf::from(env_or("HAPP_PATH", "happ/social_production.happ")),
            relay_url: env_or("RELAY_URL", &bootstrap_url),
            bootstrap_url,
            rp_id: env_or("RP_ID", "localhost"),
            rp_origin: env_or("RP_ORIGIN", "http://localhost:5173"),
            pow_bits: env_num("POW_BITS", 20),
            signups_per_ip_per_day: env_num("SIGNUPS_PER_IP_PER_DAY", 5),
            db_max_readers: env_num("DB_MAX_READERS", 4),
            restart_after_upgrade: env_or("RESTART_AFTER_UPGRADE", "1") != "0",
        }
    }
}

/// `LAIR_PASSPHRASE` unlocks every hosted agent's key. Required; losing it loses them all.
pub fn keystore_passphrase() -> anyhow::Result<Vec<u8>> {
    std::env::var("LAIR_PASSPHRASE")
        .map(String::into_bytes)
        .map_err(|_| anyhow::anyhow!("LAIR_PASSPHRASE must be set"))
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_num<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
