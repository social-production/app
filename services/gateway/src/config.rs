use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

/// Gateway settings, read once from the environment at startup.
#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    /// Conductor admin port (localhost only; never exposed).
    pub admin_port: u16,
    /// Conductor app-websocket port that `/ws` forwards to.
    pub app_ws_port: u16,
    pub happ_path: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            bind_addr: SocketAddr::from((Ipv4Addr::UNSPECIFIED, env_port("PORT", 8080))),
            admin_port: env_port("ADMIN_PORT", 4444),
            app_ws_port: env_port("APP_WS_PORT", 8888),
            happ_path: std::env::var("HAPP_PATH").map_or_else(
                |_| PathBuf::from("happ/social_production.happ"),
                PathBuf::from,
            ),
        }
    }
}

fn env_port(key: &str, default: u16) -> u16 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
