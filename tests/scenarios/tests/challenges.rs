//! M3c challenge window (docs/GOVERNANCE.md, ADR 0040): a member the proposer's snapshot left
//! out challenges the count; the decision stops applying, reopens with the larger population,
//! and applies again once enough of it votes.

use holochain::prelude::*;
use holochain::sweettest::{
    await_consistency, SweetConductor, SweetConductorBatch, SweetConductorConfig,
};
use serde::{Deserialize, Serialize};

mod common;

use common::net::{charter, Charter, Net};

/// Mirrors `project_integrity::ProposalKind::Edit` (tests can't link two integrity crates).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum ProposalKind {
    Edit {
        title: Option<String>,
        description: Option<String>,
        location: Option<String>,
    },
}

#[derive(Serialize, Debug)]
struct VoteInput {
    proposal: ActionHash,
    choice: &'static str,
}

#[derive(Deserialize, Debug)]
struct ProjectView {
    charter: Option<Charter>,
}

#[derive(Deserialize, Debug)]
struct Listed {
    id: ActionHash,
    status: String,
    population: u32,
    required: u32,
    added: u32,
    missing: u32,
    can_challenge: bool,
}

/// Three founders with bootstrap off, introduced to each other; agent 0 founded a space in the
/// commons and everyone holds its charter (joining a space needs it, ADR 0065).
async fn net_of_three(seed: &str) -> Net {
    let mut conductors =
        SweetConductorBatch::from_config_rendezvous(3, SweetConductorConfig::rendezvous(false))
            .await;
    let seed = format!("{}{seed}", common::COMMONS);
    let cells = common::install_project_with_directory(&mut conductors, 3, &seed).await;
    let mut net = Net {
        conductors,
        cells,
        space: None,
    };
    introduce(&net, &[0, 1, 2]).await;
    let space: ActionHash = net
        .call(0, "project", "create_charter", charter("productive"))
        .await;
    net.space = Some(space);
    net.settle().await;
    net
}

/// Introduce the running agents `which` to each other (bootstrap is off).
async fn introduce(net: &Net, which: &[usize]) {
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        while !SweetConductor::exchange_peer_info(which.iter().map(|&i| &net.conductors[i])).await {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("introduce agents");
}

/// Settle the two connected agents' networks.
async fn settle_pair(net: &Net) {
    let directory = [&net.cells[0].0, &net.cells[1].0];
    let project = [&net.cells[0].1, &net.cells[1].1];
    await_consistency(directory).await.expect("directory");
    await_consistency(project).await.expect("project");
}

async fn title(net: &Net, i: usize) -> String {
    let view: Option<ProjectView> = net.call(i, "project", "get_project", ()).await;
    view.and_then(|v| v.charter).expect("charter").title
}

async fn listed(net: &Net, i: usize, id: &ActionHash) -> Listed {
    let all: Vec<Listed> = net.call(i, "governance", "list_proposals", ()).await;
    all.into_iter().find(|p| &p.id == id).expect("listed")
}

async fn vote_yes(net: &Net, i: usize, proposal: &ActionHash) {
    let input = VoteInput {
        proposal: proposal.clone(),
        choice: "Yes",
    };
    let _: ActionHash = net.call(i, "governance", "vote", input).await;
}

/// Agent 2 joins while agents 0 and 1 are offline, then goes offline itself; agent 0 renames
/// the project counting only itself and agent 1, who carry it. Agent 2 is left shut down.
async fn undercounted_rename(net: &mut Net) -> ActionHash {
    let _: () = net.call(1, "project", "join", ()).await;
    net.settle().await;
    for i in [0, 1] {
        net.conductors[i].shutdown().await;
    }
    let _: () = net.call(2, "project", "join", ()).await;
    net.conductors[2].shutdown().await;
    for i in [0, 1] {
        net.conductors[i].startup().await;
    }
    introduce(net, &[0, 1]).await;
    let rename = ProposalKind::Edit {
        title: Some("Orchard".into()),
        description: None,
        location: None,
    };
    let id: ActionHash = net.call(0, "governance", "propose", rename).await;
    settle_pair(net).await;
    vote_yes(net, 1, &id).await;
    settle_pair(net).await;
    assert_eq!(title(net, 0).await, "Orchard", "passed on a count of two");
    id
}

#[tokio::test(flavor = "multi_thread")]
async fn a_challenge_reopens_a_decision_that_left_members_out() {
    let mut net = net_of_three("challenge").await;
    let id = Box::pin(undercounted_rename(&mut net)).await;

    net.conductors[2].startup().await;
    introduce(&net, &[0, 1, 2]).await;
    net.settle().await;
    let seen = listed(&net, 2, &id).await;
    assert_eq!(
        (seen.status.as_str(), seen.population, seen.missing),
        ("Passed", 2, 1)
    );
    assert!(seen.can_challenge, "within 48 hours of the decision");

    let _: ActionHash = net.call(2, "governance", "challenge", id.clone()).await;
    net.settle().await;
    let reopened = listed(&net, 0, &id).await;
    assert_eq!(
        (
            reopened.status.as_str(),
            reopened.population,
            reopened.required,
            reopened.added,
            reopened.missing,
        ),
        ("Open", 3, 3, 1, 0)
    );
    assert_eq!(
        title(&net, 0).await,
        "Garden",
        "the decision stops applying"
    );
    let again = net
        .try_call::<_, ActionHash>(1, "governance", "challenge", id.clone())
        .await;
    assert!(again.is_err(), "everyone is counted now");

    vote_yes(&net, 2, &id).await;
    net.settle().await;
    for i in 0..3 {
        assert_eq!(listed(&net, i, &id).await.status, "Passed");
        assert_eq!(
            title(&net, i).await,
            "Orchard",
            "decided again by all three"
        );
    }
    net.shut_down().await;
}
