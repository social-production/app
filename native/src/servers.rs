//! The bootstrap/relay servers this device can use, and which one it picked (ADR 0091).
//!
//! Order: the one the person saved in Settings, then the shipped list. At start the app asks
//! each for `/health` (in parallel, a few seconds each) and uses the first that answers; if
//! none does, the first one, so the node retries it. `SP_BOOTSTRAP_URL` replaces the whole
//! list (local dev, CI).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use serde::Serialize;
use tauri_plugin_hc::{UserNetworkConfig, UserNetworkConfigPath};

/// One server: kitsune2's bootstrap and the iroh relay, usually at the same URL.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub bootstrap: String,
    pub relay: String,
}

impl Server {
    fn same(url: &str) -> Self {
        Self {
            bootstrap: url.into(),
            relay: url.into(),
        }
    }
}

/// Shipped servers, in order. Railway's (node #1) first. Holochain's public development
/// bootstrap and n0's iroh relay are the last resort (docs/plans/SEEDLESS.md): if Railway is
/// gone, every desktop falls through to them and they still find each other. Community copies
/// (docs/RUN-A-BOOTSTRAP.md) go in between once someone runs one.
pub const SHIPPED: &[(&str, &str)] = &[
    (
        "https://bootstrap-production-f978.up.railway.app",
        "https://bootstrap-production-f978.up.railway.app",
    ),
    (
        "https://dev-test-bootstrap2.holochain.org",
        "https://use1-1.relay.n0.iroh-canary.iroh.link./",
    ),
];

const PROBE_TIMEOUT: Duration = Duration::from_secs(4);

/// What Settings shows: the server in use, the one the person saved, and the shipped list.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Servers {
    pub current: Server,
    pub saved: Option<Server>,
    pub shipped: Vec<Server>,
    /// True when `SP_BOOTSTRAP_URL` set the list (Settings can't change it then).
    pub fixed: bool,
}

static CHOSEN: OnceLock<Servers> = OnceLock::new();

/// The file Settings saves to (the plugin's `set_user_network_config` writes it).
pub fn saved_path(data_dir: &Path) -> PathBuf {
    data_dir.join("user-network-config.json")
}

/// The managed state the plugin's network commands need.
pub fn managed_path(data_dir: &Path) -> UserNetworkConfigPath {
    UserNetworkConfigPath(saved_path(data_dir))
}

/// Pick the server to boot with, and remember the choice for Settings.
pub fn choose(data_dir: &Path) -> Server {
    let (candidates, saved, fixed) = candidates(data_dir);
    let current = first_healthy(&candidates).unwrap_or_else(|| candidates[0].clone());
    log::info!(
        "network: using {} (of {})",
        current.bootstrap,
        candidates.len()
    );
    let _ = CHOSEN.set(Servers {
        current: current.clone(),
        saved,
        shipped: shipped(),
        fixed,
    });
    current
}

fn shipped() -> Vec<Server> {
    SHIPPED
        .iter()
        .map(|(bootstrap, relay)| Server {
            bootstrap: (*bootstrap).into(),
            relay: (*relay).into(),
        })
        .collect()
}

fn candidates(data_dir: &Path) -> (Vec<Server>, Option<Server>, bool) {
    if let Ok(url) = std::env::var("SP_BOOTSTRAP_URL") {
        return (vec![Server::same(&url)], None, true);
    }
    let saved = match UserNetworkConfig::read(&saved_path(data_dir)) {
        Ok(Some(c)) => c.bootstrap_url.map(|b| {
            let bootstrap = b.as_str().trim_end_matches('/').to_string();
            let relay = c
                .relay_url
                .map_or_else(|| bootstrap.clone(), |r| r.as_str().to_string());
            Server { bootstrap, relay }
        }),
        Ok(None) => None,
        Err(e) => {
            log::warn!("ignoring saved network settings: {e}");
            None
        }
    };
    (ordered(saved.clone(), shipped()), saved, false)
}

/// The saved server first, then the shipped ones, without repeats.
pub fn ordered(saved: Option<Server>, shipped: Vec<Server>) -> Vec<Server> {
    let mut list: Vec<Server> = saved.into_iter().collect();
    for server in shipped {
        if !list.iter().any(|s| s.bootstrap == server.bootstrap) {
            list.push(server);
        }
    }
    list
}

/// The first server, in order, whose bootstrap answers `/health`. All are asked at once, so
/// a dead server costs at most one timeout.
fn first_healthy(list: &[Server]) -> Option<Server> {
    let answers: Vec<bool> = std::thread::scope(|scope| {
        let probes: Vec<_> = list
            .iter()
            .map(|s| scope.spawn(move || healthy(&s.bootstrap)))
            .collect();
        probes
            .into_iter()
            .map(|p| p.join().unwrap_or(false))
            .collect()
    });
    list.iter()
        .zip(answers)
        .find(|(_, ok)| *ok)
        .map(|(s, _)| s.clone())
}

fn healthy(bootstrap: &str) -> bool {
    let url = format!("{}/health", bootstrap.trim_end_matches('/'));
    let client = match reqwest::blocking::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("network: no HTTP client: {e}");
            return false;
        }
    };
    match client.get(&url).send() {
        Ok(r) if r.status().is_success() => true,
        Ok(r) => {
            log::warn!("network: {url} answered {}", r.status());
            false
        }
        Err(e) => {
            log::warn!("network: {url} unreachable: {e}");
            false
        }
    }
}

/// Settings: which servers this device knows and which one it is using.
#[tauri::command]
pub fn network_servers() -> Option<Servers> {
    CHOSEN.get().cloned()
}

/// Settings: forget the saved server and restart on the shipped list.
#[tauri::command]
pub fn reset_network_servers(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let path = app.state::<UserNetworkConfigPath>().0.clone();
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    app.request_restart();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_comes_first_without_repeats() {
        let shipped = shipped();
        let saved = Server::same("https://bootstrap.example.org");
        let list = ordered(Some(saved.clone()), shipped.clone());
        assert_eq!(list[0], saved);
        assert_eq!(list.len(), shipped.len() + 1);

        let again = ordered(Some(shipped[1].clone()), shipped.clone());
        assert_eq!(again[0], shipped[1]);
        assert_eq!(again.len(), shipped.len());
    }

    #[test]
    fn no_saved_is_the_shipped_list() {
        assert_eq!(ordered(None, shipped()), shipped());
    }
}
