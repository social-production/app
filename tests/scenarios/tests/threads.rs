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

/// Mirrors `discussion_integrity::Attachment`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Attachment {
    kind: String,
    name: String,
    mime: String,
    size: u32,
    chunks: Vec<ActionHash>,
}

#[derive(Deserialize, Debug)]
struct MessageView {
    body: String,
}

async fn photo(net: &Net, i: usize, name: &str) -> Attachment {
    let bytes = serde_bytes::ByteBuf::from(vec![7u8; 2048]);
    let chunk: ActionHash = net.call(i, "discussion", "upload_chunk", bytes).await;
    Attachment {
        kind: "image".into(),
        name: name.into(),
        mime: "image/jpeg".into(),
        size: 2048,
        chunks: vec![chunk],
    }
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

/// ADR 0101: a post's author adds its photos once; everyone reads them back, and they are not
/// one of the post's comments.
#[tokio::test(flavor = "multi_thread")]
async fn a_post_carries_its_authors_photos() {
    let post = Charter {
        title: "Planting garlic".into(),
        description: "Planting garlic this weekend.".into(),
        ..charter("post")
    };
    let net = Net::with_charter(2, 1, "post-photos", post).await;

    let theirs = photo(&net, 1, "not-mine.jpg").await;
    let refused: Result<ActionHash, String> = net
        .try_call(1, "discussion", "set_post_media", vec![theirs])
        .await;
    assert!(refused.is_err(), "only the author adds photos");

    let beds = photo(&net, 0, "beds.jpg").await;
    let soil = photo(&net, 0, "soil.jpg").await;
    let _: ActionHash = net
        .call(
            0,
            "discussion",
            "set_post_media",
            vec![beds.clone(), soil.clone()],
        )
        .await;
    let mine: Vec<Attachment> = net.call(0, "discussion", "post_media", ()).await;
    assert_eq!(mine, vec![beds.clone(), soil.clone()], "seen at once");
    let again: Result<ActionHash, String> = net
        .try_call(0, "discussion", "set_post_media", vec![beds.clone()])
        .await;
    assert!(again.is_err(), "photos are added once");

    let _: ActionHash = net
        .call(1, "discussion", "post_message", comment("Lovely.", None))
        .await;
    net.settle().await;
    let seen: Vec<Attachment> = net.call(1, "discussion", "post_media", ()).await;
    assert_eq!(seen, vec![beds, soil], "the reader gets the photos");
    for i in 0..2 {
        let comments: Vec<MessageView> = net.call(i, "discussion", "list_messages", ()).await;
        let bodies: Vec<&str> = comments.iter().map(|m| m.body.as_str()).collect();
        assert_eq!(bodies, ["Lovely."], "the photos are not a comment");
    }
    net.shut_down().await;
}
