//! Bootstrap servers other desktops announced in the directory (ADR 0094). The UI reads them
//! once connected (`holo/helping.ts`) and hands them here; they are saved for the next start,
//! when `servers.rs` tries them after Railway's and before Holochain's public ones.
//!
//! Anyone can announce, so each note is checked here, and an announced server is only used
//! when the ones before it don't answer. A bad server can hide peers from a new install,
//! but it cannot forge them: peer records are signed by their agents.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::helping::public_origin;
use crate::servers::Server;

/// At most this many announced servers are kept (the newest first).
const KEEP: usize = 8;

fn path(data_dir: &Path) -> PathBuf {
    data_dir.join("announced-servers.json")
}

#[derive(Deserialize)]
struct Note {
    bootstrap: String,
    relay: String,
}

/// The valid servers in `notes` (newest first, as the zome lists them), without repeats.
pub fn parse(notes: &[String]) -> Vec<Server> {
    let mut servers: Vec<Server> = Vec::new();
    for note in notes {
        let Ok(Note { bootstrap, relay }) = serde_json::from_str::<Note>(note) else {
            continue;
        };
        let (Ok(bootstrap), Ok(relay)) = (public_origin(&bootstrap), public_origin(&relay)) else {
            continue;
        };
        if servers.iter().all(|s| s.bootstrap != bootstrap) {
            servers.push(Server { bootstrap, relay });
        }
        if servers.len() == KEEP {
            break;
        }
    }
    servers
}

/// The servers saved at the last connection.
pub fn read(data_dir: &Path) -> Vec<Server> {
    let notes: Vec<String> = std::fs::read(path(data_dir))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    parse(&notes)
}

/// The UI, once connected: the notes the directory lists, newest first.
#[tauri::command]
pub fn remember_servers(app: tauri::AppHandle, notes: Vec<String>) -> Result<usize, String> {
    use tauri::Manager;
    let servers = parse(&notes);
    let keep: Vec<String> = servers
        .iter()
        .filter_map(|s| {
            let note = serde_json::json!({ "bootstrap": s.bootstrap, "relay": s.relay });
            serde_json::to_string(&note).ok()
        })
        .collect();
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let json = serde_json::to_vec(&keep).map_err(|e| e.to_string())?;
    std::fs::write(path(&dir.join("holochain")), json).map_err(|e| e.to_string())?;
    Ok(servers.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(bootstrap: &str, relay: &str) -> String {
        serde_json::json!({ "bootstrap": bootstrap, "relay": relay }).to_string()
    }

    #[test]
    fn keeps_valid_notes_once_each() {
        let notes = vec![
            note("http://203.0.113.5:8787", "http://203.0.113.5:8787"),
            "not json".into(),
            note("http://localhost:8787", "http://localhost:8787"),
            note("https://boot.example.org/", "https://boot.example.org"),
            note("http://203.0.113.5:8787/", "http://203.0.113.5:8787"),
            note("javascript:alert(1)", "https://boot.example.org"),
        ];
        let servers = parse(&notes);
        let urls: Vec<&str> = servers.iter().map(|s| s.bootstrap.as_str()).collect();
        assert_eq!(
            urls,
            ["http://203.0.113.5:8787", "https://boot.example.org"]
        );
    }

    #[test]
    fn keeps_at_most_eight() {
        let notes: Vec<String> = (1..=12)
            .map(|i| {
                note(
                    &format!("http://203.0.113.{i}:8787"),
                    "https://r.example.org",
                )
            })
            .collect();
        assert_eq!(parse(&notes).len(), KEEP);
    }
}
