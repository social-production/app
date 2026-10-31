//! Threads and posts (ADR 0070): commons spaces with mode `thread` or `post` and no lifecycle.
//! Their comments are the space's chat, nested by `reply_to`, and anyone votes a comment up or
//! down (one vote each; changing it replaces the old one).

use holochain::prelude::*;
use serde::{Deserialize, Serialize};

mod common;

use common::net::{charter, Charter, Net};

#[derive(Serialize, Debug)]
struct PostInput {
    body: String,
    reply_to: Option<ActionHash>,
}

#[derive(Serialize, Debug)]
struct VoteInput {
    message: ActionHash,
    direction: i8,
}

#[derive(Deserialize, Debug)]
struct VoteView {
    id: ActionHash,
    count: i32,
    mine: i8,
}

#[derive(Deserialize, Debug)]
struct PhaseStep {}

#[derive(Deserialize, Debug)]
struct ProjectView {
    stage: String,
    phases: Vec<PhaseStep>,
    next_phase: Option<PhaseStep>,
}

/// Mirrors the update arm of `project_integrity::ProposalKind`.
#[derive(Serialize, Debug)]
enum Kind {
    Update { body: String },
}

fn thread() -> Charter {
    Charter {
        title: "Which seeds for spring?".into(),
        description: "Share what grew well last year.".into(),
        tags: vec!["#food".into()],
        ..charter("thread")
    }
}

fn comment(body: &str, reply_to: Option<&ActionHash>) -> PostInput {
    PostInput {
        body: body.into(),
        reply_to: reply_to.cloned(),
    }
}

async fn vote(net: &Net, i: usize, message: &ActionHash, direction: i8) {
    let (): () = net
        .call(
            i,
            "discussion",
            "vote_message",
            VoteInput {
                message: message.clone(),
                direction,
            },
        )
        .await;
}

async fn votes(net: &Net, i: usize, ids: &[&ActionHash]) -> Vec<VoteView> {
    let ids: Vec<ActionHash> = ids.iter().map(|&h| h.clone()).collect();
    net.call(i, "discussion", "message_votes", ids).await
}

#[tokio::test(flavor = "multi_thread")]
async fn comments_nest_and_take_one_vote_each() {
    let net = Net::with_charter(3, 1, "threads", thread()).await;
    let view: ProjectView = net.call(0, "project", "get_project", ()).await;
    assert_eq!(view.stage, "Thread");
    assert!(
        view.phases.is_empty() && view.next_phase.is_none(),
        "no lifecycle"
    );
    let proposal: Result<ActionHash, String> = net
        .try_call(
            0,
            "governance",
            "propose",
            Kind::Update {
                body: "Edit".into(),
            },
        )
        .await;
    assert!(proposal.is_err(), "threads take no proposals");

    let top: ActionHash = net
        .call(1, "discussion", "post_message", comment("Tomatoes!", None))
        .await;
    net.settle().await;
    let reply: ActionHash = net
        .call(
            0,
            "discussion",
            "post_message",
            comment("Which kind?", Some(&top)),
        )
        .await;
    net.settle().await;

    vote(&net, 0, &top, 1).await;
    vote(&net, 2, &top, 1).await;
    vote(&net, 1, &reply, -1).await;
    // Changing your vote replaces it; 0 takes it back.
    vote(&net, 2, &top, -1).await;
    vote(&net, 0, &reply, 1).await;
    vote(&net, 0, &reply, 0).await;
    net.settle().await;

    let seen = votes(&net, 2, &[&top, &reply]).await;
    assert_eq!(seen[0].id, top);
    assert_eq!((seen[0].count, seen[0].mine), (0, -1), "one up, one down");
    assert_eq!((seen[1].count, seen[1].mine), (-1, 0), "ann took hers back");
    let ann = votes(&net, 0, &[&top]).await;
    assert_eq!(ann[0].mine, 1);

    // Integrity: a vote is on a comment, not on any record (here the charter).
    let space = net.space.clone().expect("a commons space");
    let odd: Result<(), String> = net
        .try_call(
            0,
            "discussion",
            "vote_message",
            VoteInput {
                message: space,
                direction: 1,
            },
        )
        .await;
    assert!(odd.is_err(), "only comments take votes");
    net.shut_down().await;
}
