#![allow(dead_code)] // each test binary uses a different subset

use std::path::PathBuf;

use holo_hash::AgentPubKeyB64;
use holochain::prelude::*;

pub mod chats;
pub mod links;
pub mod net;
pub mod plans;
use holochain::sweettest::{SweetAgents, SweetCell, SweetConductorBatch, SweetDnaFile};
use identity_integrity::{DirectoryProperties, Founder};

/// Shut every conductor down, drop them, and wait until their databases have closed. Call it
/// at the end of each test. `shutdown()` keeps a conductor's state (its `SQLite` pools included)
/// for a restart, and the pools' worker threads (`sqlx-sqlite-worker-*`) wind down for tens of
/// seconds after the conductors are dropped. If the last test returned first, the process exited
/// while those threads were still inside `SQLite`: heap-corruption aborts and SIGSEGVs at exit in
/// CI, after every test had passed.
pub async fn shut_down(mut conductors: SweetConductorBatch) {
    for i in 0..conductors.len() {
        if conductors[i].is_running() {
            conductors[i].shutdown().await;
        }
    }
    drop(conductors);
    databases_closed().await;
}

/// Wait (at most two minutes) until no `SQLite` worker thread is left in this process. Tests run
/// one at a time (`--test-threads=1`), so every such thread belongs to the conductors just
/// dropped.
async fn databases_closed() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_mins(2);
    while sqlite_workers() > 0 {
        if std::time::Instant::now() > deadline {
            eprintln!(
                "warning: {} SQLite worker threads still running",
                sqlite_workers()
            );
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
}

/// Live threads named `sqlx-sqlite-worker-*` (Linux truncates names to 15 bytes).
fn sqlite_workers() -> usize {
    let Ok(tasks) = std::fs::read_dir("/proc/self/task") else {
        return 0; // not Linux: nothing to wait for that we can see
    };
    tasks
        .flatten()
        .filter(|t| {
            std::fs::read_to_string(t.path().join("comm"))
                .is_ok_and(|name| name.starts_with("sqlx-sqlite-wor"))
        })
        .count()
}

fn dna_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../dnas")
        .join(name)
        .join(format!("sp_{name}.dna"))
}

/// Load a packed DNA from `dnas/<name>/sp_<name>.dna` (built by scripts/build-happ.sh).
pub async fn load_dna(name: &str) -> DnaFile {
    let path = dna_path(name);
    SweetDnaFile::from_bundle(&path)
        .await
        .unwrap_or_else(|e| panic!("load {}: {e} — run scripts/build-happ.sh", path.display()))
}

/// The packed DNA `name` with a network seed (a clone, as `createCloneCell` makes one).
pub async fn dna_with(name: &str, seed: String) -> DnaFile {
    let modifiers = DnaModifiersOpt::<SerializedBytes>::none().with_network_seed(seed);
    SweetDnaFile::from_bundle_with_overrides(&dna_path(name), modifiers)
        .await
        .unwrap_or_else(|e| panic!("load {name}: {e}"))
}

/// The directory DNA with the given agents as founders (a different network per list).
pub async fn directory_with_founders(founders: &[AgentPubKey]) -> DnaFile {
    let props = DirectoryProperties {
        kind: "directory".into(),
        founders: founders
            .iter()
            .enumerate()
            .map(|(i, agent)| Founder {
                name: format!("founder {i}"),
                agent: AgentPubKeyB64::from(agent.clone()).to_string(),
            })
            .collect(),
    };
    let modifiers = DnaModifiersOpt::none().with_properties(props);
    SweetDnaFile::from_bundle_with_overrides(&dna_path("directory"), modifiers)
        .await
        .expect("load directory dna")
}

/// Seeds starting with this make a commons cell (no creator in the seed, ADR 0065), the way
/// the base cell (no seed) is the commons in the app; a seed still keeps tests apart.
pub const COMMONS: &str = "commons.";

/// The seed a project cell gets: the commons' own, or a clone's named after its creator.
fn project_seed(creator: &AgentPubKey, seed: &str) -> String {
    if seed.starts_with(COMMONS) {
        seed.to_string()
    } else {
        format!("{}.{seed}", AgentPubKeyB64::from(creator.clone()))
    }
}

/// One agent per conductor; the first `founders` agents are founders. Returns the
/// directory cells in conductor order.
pub async fn directory_network(
    conductors: &mut SweetConductorBatch,
    founders: usize,
) -> Vec<SweetCell> {
    let mut agents = Vec::new();
    for i in 0..conductors.len() {
        agents.push(SweetAgents::one(conductors[i].keystore()).await);
    }
    let dna = directory_with_founders(&agents[..founders]).await;
    let mut cells = Vec::new();
    for (i, agent) in agents.into_iter().enumerate() {
        let app = conductors[i]
            .setup_app_for_agent("sp", agent, std::slice::from_ref(&dna))
            .await
            .expect("install app");
        cells.push(app.into_cells().remove(0));
    }
    conductors.exchange_peer_info().await;
    cells
}

/// One project cell per conductor's agent: a clone named after the first agent (seed
/// `<first agent>.<seed>`), or a commons cell when `seed` starts with `COMMONS`. Returns the
/// cells in conductor order.
pub async fn project_network(conductors: &mut SweetConductorBatch, seed: &str) -> Vec<SweetCell> {
    let mut agents = Vec::new();
    for i in 0..conductors.len() {
        agents.push(SweetAgents::one(conductors[i].keystore()).await);
    }
    // A clone's creator is named in the seed (project_integrity::creator_in_seed).
    let modifiers =
        DnaModifiersOpt::<SerializedBytes>::none().with_network_seed(project_seed(&agents[0], seed));
    let dna = SweetDnaFile::from_bundle_with_overrides(&dna_path("project"), modifiers)
        .await
        .expect("load project dna");
    let mut cells = Vec::new();
    for (i, agent) in agents.into_iter().enumerate() {
        let app = conductors[i]
            .setup_app_for_agent("sp", agent, std::slice::from_ref(&dna))
            .await
            .expect("install app");
        cells.push(app.into_cells().remove(0));
    }
    conductors.exchange_peer_info().await;
    cells
}

/// Each agent gets one app with both roles, named as in happ.yaml ("directory", "project"):
/// the directory with the first `founders` agents as founders, and one project cell (a clone
/// created by the first agent, or a commons cell, as `project_network`). Returns `(directory, project)` cells per conductor.
pub async fn project_with_directory(
    conductors: &mut SweetConductorBatch,
    founders: usize,
    seed: &str,
) -> Vec<(SweetCell, SweetCell)> {
    let cells = install_project_with_directory(conductors, founders, seed).await;
    conductors.exchange_peer_info().await;
    cells
}

/// `project_with_directory` without introducing the peers, for tests that keep someone offline
/// (with bootstrap disabled, peers meet only through `exchange_peer_info`).
pub async fn install_project_with_directory(
    conductors: &mut SweetConductorBatch,
    founders: usize,
    seed: &str,
) -> Vec<(SweetCell, SweetCell)> {
    let mut agents = Vec::new();
    for i in 0..conductors.len() {
        agents.push(SweetAgents::one(conductors[i].keystore()).await);
    }
    let directory = directory_with_founders(&agents[..founders]).await;
    let modifiers =
        DnaModifiersOpt::<SerializedBytes>::none().with_network_seed(project_seed(&agents[0], seed));
    let project = SweetDnaFile::from_bundle_with_overrides(&dna_path("project"), modifiers)
        .await
        .expect("load project dna");
    let mut out = Vec::new();
    for (i, agent) in agents.into_iter().enumerate() {
        let roles = [
            ("directory".to_string(), directory.clone()),
            ("project".to_string(), project.clone()),
        ];
        let app = conductors[i]
            .setup_app_for_agent("sp", agent, &roles)
            .await
            .expect("install app");
        let (dir, proj): (Vec<SweetCell>, Vec<SweetCell>) = app
            .into_cells()
            .into_iter()
            .partition(|c| c.cell_id().dna_hash() == directory.dna_hash());
        let (Some(d), Some(p)) = (dir.into_iter().next(), proj.into_iter().next()) else {
            panic!("app has both roles");
        };
        out.push((d, p));
    }
    out
}
