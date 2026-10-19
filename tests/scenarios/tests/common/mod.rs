#![allow(dead_code)] // each test binary uses a different subset

use std::path::PathBuf;

use holo_hash::AgentPubKeyB64;
use holochain::prelude::*;
use holochain::sweettest::{SweetAgents, SweetCell, SweetConductorBatch, SweetDnaFile};
use identity_integrity::{DirectoryProperties, Founder};

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

/// One project clone (seed `<first agent>.<seed>`) installed for each conductor's agent,
/// as the gateway and UI do with `createCloneCell`. Returns the cells in conductor order.
pub async fn project_network(conductors: &mut SweetConductorBatch, seed: &str) -> Vec<SweetCell> {
    let mut agents = Vec::new();
    for i in 0..conductors.len() {
        agents.push(SweetAgents::one(conductors[i].keystore()).await);
    }
    // The creator is named in the seed (project_integrity::creator_in_seed).
    let creator = AgentPubKeyB64::from(agents[0].clone());
    let modifiers =
        DnaModifiersOpt::<SerializedBytes>::none().with_network_seed(format!("{creator}.{seed}"));
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
/// the directory with the first `founders` agents as founders, and one project clone created by
/// the first agent. Returns `(directory, project)` cells per conductor.
pub async fn project_with_directory(
    conductors: &mut SweetConductorBatch,
    founders: usize,
    seed: &str,
) -> Vec<(SweetCell, SweetCell)> {
    let mut agents = Vec::new();
    for i in 0..conductors.len() {
        agents.push(SweetAgents::one(conductors[i].keystore()).await);
    }
    let directory = directory_with_founders(&agents[..founders]).await;
    let creator = AgentPubKeyB64::from(agents[0].clone());
    let modifiers =
        DnaModifiersOpt::<SerializedBytes>::none().with_network_seed(format!("{creator}.{seed}"));
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
    conductors.exchange_peer_info().await;
    out
}
