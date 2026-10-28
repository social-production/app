//! Phase 0 of the commons (ADR 0065, `docs/plans/COMMONS.md`): what each project costs a node
//! when every project is its own network (a clone). Ignored in CI; run it on purpose:
//!   cargo test -p scenarios --test scale -- --ignored --nocapture
//! It creates projects one clone at a time on a single conductor and prints the process's
//! resident memory, threads and open files after each step.

use holo_hash::AgentPubKeyB64;
use holochain::prelude::*;
use holochain::sweettest::SweetConductorBatch;

mod common;

use common::net::charter;

/// Steps (total projects) at which the process is measured; `SCALE_MAX` caps them.
const STEPS: [usize; 5] = [1, 25, 50, 100, 200];

fn status(field: &str) -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    status
        .lines()
        .find(|l| l.starts_with(field))
        .and_then(|l| l.split_whitespace().nth(1)?.parse().ok())
        .unwrap_or(0)
}

fn sample(projects: usize) {
    let files = std::fs::read_dir("/proc/self/fd").map_or(0, Iterator::count);
    println!(
        "projects={projects} rss_mb={} threads={} open_files={files}",
        status("VmRSS:") / 1024,
        status("Threads:")
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "a measurement, not a check (ADR 0065 Phase 0)"]
async fn each_project_clone_costs_the_node() {
    let max: usize = std::env::var("SCALE_MAX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    let mut conductors = SweetConductorBatch::standard(1).await;
    let cells = common::project_with_directory(&mut conductors, 1, "scale-0").await;
    let agent = cells[0].1.agent_pubkey().clone();
    let creator = AgentPubKeyB64::from(agent);
    sample(0);
    let mut made = 0;
    for &step in STEPS.iter().filter(|s| **s <= max) {
        while made < step {
            let payload = CreateCloneCellPayload {
                role_name: "project".into(),
                modifiers: DnaModifiersOpt::none()
                    .with_network_seed(format!("{creator}.scale{made}")),
                membrane_proof: None,
                name: None,
            };
            let clone = conductors[0]
                .raw_handle()
                .create_clone_cell(&"sp".to_string(), payload)
                .await
                .expect("clone");
            let zome = conductors[0]
                .get_sweet_cell(clone.cell_id)
                .expect("clone cell")
                .zome("project");
            let _: ActionHash = conductors[0]
                .call(&zome, "create_charter", charter("productive"))
                .await;
            made += 1;
        }
        // Let the new networks settle (gossip, publish) before measuring.
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        sample(made);
    }
    common::shut_down(conductors).await;
}
