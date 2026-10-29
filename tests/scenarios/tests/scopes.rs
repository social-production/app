//! Channels and communities (ADR 0068): spaces whose title is their handle and subtype their
//! join policy. Public ones live in the commons; a closed community needs a private seed.
//! Members vote people out; nothing renames a scope, and only scopes remove members.

use holochain::prelude::*;
use serde::{Deserialize, Serialize};

mod common;

use common::net::{charter, Charter, Net};

#[derive(Serialize, Debug)]
enum ProposalKind {
    Edit {
        title: Option<String>,
        description: Option<String>,
        location: Option<String>,
    },
    RemoveMember {
        agent: AgentPubKey,
    },
}

#[derive(Serialize, Debug)]
enum VoteChoice {
    Yes,
}

#[derive(Serialize, Debug)]
struct VoteInput {
    proposal: ActionHash,
    choice: VoteChoice,
}

#[derive(Deserialize, Debug)]
struct ProjectView {
    members: Vec<AgentPubKey>,
}

fn scope(mode: &str, handle: &str, policy: &str) -> Charter {
    Charter {
        title: handle.into(),
        description: "Growing food together.".into(),
        subtype: policy.into(),
        ..charter(mode)
    }
}

fn agent(net: &Net, i: usize) -> AgentPubKey {
    net.cells[i].1.agent_pubkey().clone()
}

async fn members(net: &Net, i: usize) -> Vec<AgentPubKey> {
    let view: Option<ProjectView> = net.call(i, "project", "get_project", ()).await;
    view.expect("the channel").members
}

#[tokio::test(flavor = "multi_thread")]
async fn members_vote_someone_out_of_a_channel() {
    // Agents 0 and 1 are founders (Full); agent 2 is New. Agent 0 creates #food.
    let net = Net::with_charter(3, 2, "scopes", scope("channel", "Food", "open")).await;
    assert_eq!(members(&net, 0).await.len(), 3);

    for bad in [
        scope("channel", "Food", "closed"),
        scope("channel", "Platform", "open"),
        scope("community", "Two words", "open"),
        scope("community", "Brunswick", "closed"),
    ] {
        let refused: Result<ActionHash, String> =
            net.try_call(0, "project", "create_charter", bad).await;
        assert!(refused.is_err(), "refused in the commons: {refused:?}");
    }
    let rename = ProposalKind::Edit {
        title: Some("Food2".into()),
        description: None,
        location: None,
    };
    let renamed: Result<ActionHash, String> =
        net.try_call(0, "governance", "propose", rename).await;
    assert!(renamed.is_err(), "a handle stays");

    let out = ProposalKind::RemoveMember {
        agent: agent(&net, 2),
    };
    let proposal: ActionHash = net.call(0, "governance", "propose", out).await;
    net.settle().await;
    let vote = VoteInput {
        proposal,
        choice: VoteChoice::Yes,
    };
    let _: ActionHash = net.call(1, "governance", "vote", vote).await;
    net.settle().await;
    let left = members(&net, 0).await;
    assert_eq!(left.len(), 2, "removed: {left:?}");
    assert!(!left.contains(&agent(&net, 2)));
    net.shut_down().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_closed_community_lives_in_its_own_network() {
    let net = Net::with_charter(
        2,
        1,
        "private.community",
        scope("community", "Brunswick", "closed"),
    )
    .await;
    assert_eq!(members(&net, 1).await.len(), 2);
    let open: Result<ActionHash, String> = net
        .try_call(
            0,
            "project",
            "create_charter",
            scope("community", "Coburg", "open"),
        )
        .await;
    assert!(open.is_err(), "only closed communities in a private network");
    net.shut_down().await;
}
