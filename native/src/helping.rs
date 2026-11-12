//! "Help the network" (SEEDLESS step 3b, ADR 0094). When the person turns it on in
//! Settings → Network, this desktop runs Kitsune's bootstrap server (with its iroh relay) on a
//! port they forwarded, and the UI announces the public address in the directory
//! (`holo/helping.ts`, the health zome's `announce_server`). Other apps add announced servers
//! to their list (`announced.rs`), so new installs can still find people if Railway's
//! bootstrap is gone.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use kitsune2_bootstrap_srv::{BootstrapSrv, Config};
use serde::{Deserialize, Serialize};

use crate::servers::Server;

/// The port suggested in Settings (any free port works; the person forwards it).
pub const DEFAULT_PORT: u16 = 8787;

/// What the person chose in Settings.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelpSettings {
    pub enabled: bool,
    pub port: u16,
    /// How other people reach this server: `http(s)://host[:port]`.
    pub public_url: String,
}

/// What Settings shows about it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelpState {
    pub settings: HelpSettings,
    pub running: bool,
    pub error: Option<String>,
    /// The server to announce in the directory, while it runs.
    pub announce: Option<Server>,
}

struct Helping {
    state: HelpState,
    /// Kept alive for the app's lifetime: dropping it stops the server.
    _server: Mutex<Option<BootstrapSrv>>,
}

static HELPING: OnceLock<Helping> = OnceLock::new();

fn settings_path(data_dir: &Path) -> PathBuf {
    data_dir.join("help-network.json")
}

/// The saved choice (off, on the default port, when nothing is saved).
pub fn read(data_dir: &Path) -> HelpSettings {
    std::fs::read(settings_path(data_dir))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or(HelpSettings {
            port: DEFAULT_PORT,
            ..HelpSettings::default()
        })
}

/// `http(s)://host[:port]`, without a path: the address announced to everyone.
pub fn public_origin(url: &str) -> Result<String, String> {
    let parsed = url2::Url2::try_parse(url.trim()).map_err(|_| bad_address())?;
    let path_ok = matches!(parsed.path(), "" | "/");
    let scheme_ok = matches!(parsed.scheme(), "http" | "https");
    let host = parsed.host_str().unwrap_or_default();
    if !scheme_ok || !path_ok || host.is_empty() || parsed.query().is_some() {
        return Err(bad_address());
    }
    if host == "localhost" || host.starts_with("127.") || host == "[::1]" {
        return Err("Use the address other people reach, not this computer's own.".into());
    }
    Ok(parsed.as_str().trim_end_matches('/').to_string())
}

fn bad_address() -> String {
    "The address must look like http://203.0.113.5:8787 or https://boot.example.org".into()
}

/// Start the server if the person turned it on. Returns this node's own entry for the list
/// (`servers.rs` puts it after Railway's, so the node uses its own server when Railway's is
/// gone).
pub fn start(data_dir: &Path) -> Option<Server> {
    let settings = read(data_dir);
    let (server, error) = if settings.enabled {
        match public_origin(&settings.public_url).and_then(|_| serve(settings.port)) {
            Ok(server) => (Some(server), None),
            Err(e) => {
                log::error!("help the network: {e}");
                (None, Some(e))
            }
        }
    } else {
        (None, None)
    };
    let running = server.is_some();
    let announce = running.then(|| {
        let url = public_origin(&settings.public_url).unwrap_or_default();
        Server {
            bootstrap: url.clone(),
            relay: url,
        }
    });
    let local = announce
        .as_ref()
        .map(|public| own(settings.port, &public.relay));
    let _ = HELPING.set(Helping {
        state: HelpState {
            settings,
            running,
            error,
            announce,
        },
        _server: Mutex::new(server),
    });
    local
}

/// This node registers with its own bootstrap directly, but uses its relay by the public
/// address: peers reach a node through the relay its peer record names, so that address must
/// work from outside (and from here, which most routers allow).
fn own(port: u16, public_relay: &str) -> Server {
    Server {
        bootstrap: format!("http://127.0.0.1:{port}"),
        relay: public_relay.into(),
    }
}

/// Kitsune's bootstrap server and its relay on every IPv4 address at `port` (0: any free
/// one). It starts its own threads and runtime; started from a plain thread, since it
/// blocks on its runtime while binding.
fn serve(port: u16) -> Result<BootstrapSrv, String> {
    let mut config = Config::production();
    config.worker_thread_count = 2;
    config.listen_address_list = vec![SocketAddr::from((Ipv4Addr::UNSPECIFIED, port))];
    // Address discovery over QUIC needs a certificate; plain HTTP relays work without it.
    config.quic_bind_addr = None;
    std::thread::spawn(move || BootstrapSrv::new(config))
        .join()
        .map_err(|_| "the bootstrap server crashed while starting".to_string())?
        .map_err(|e| format!("could not open port {port}: {e}"))
}

/// Settings: whether this desktop helps, and how it is going.
#[tauri::command]
pub fn help_network_state() -> Option<HelpState> {
    HELPING.get().map(|h| h.state.clone())
}

/// Settings: save the choice and restart the app to apply it.
#[tauri::command]
pub fn set_help_network(app: tauri::AppHandle, settings: HelpSettings) -> Result<(), String> {
    use tauri::Manager;
    let mut settings = settings;
    if settings.enabled {
        settings.public_url = public_origin(&settings.public_url)?;
        if settings.port == 0 {
            return Err("Choose the port you forwarded (1-65535).".into());
        }
    }
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = settings_path(&dir.join("holochain"));
    let json = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    app.request_restart();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_addresses_are_origins_others_can_reach() {
        assert_eq!(
            public_origin(" http://203.0.113.5:8787/ "),
            Ok("http://203.0.113.5:8787".into())
        );
        assert_eq!(
            public_origin("https://boot.example.org"),
            Ok("https://boot.example.org".into())
        );
        for bad in [
            "boot.example.org",
            "ftp://boot.example.org",
            "https://boot.example.org/bootstrap",
            "http://localhost:8787",
            "http://127.0.0.1:8787",
            "https://boot.example.org/?x=1",
        ] {
            assert!(public_origin(bad).is_err(), "{bad} should be refused");
        }
    }

    /// The server starts on a free port and answers `/health`, as Railway's does.
    #[test]
    fn serves_a_bootstrap_that_answers_health() {
        let server = serve(0).expect("server starts");
        let port = server.listen_addrs()[0].port();
        let url = format!("http://127.0.0.1:{port}/health");
        let client = reqwest::blocking::Client::builder().no_proxy().build();
        let answer = client.expect("client").get(url).send().expect("answers");
        assert!(answer.status().is_success());
    }
}
