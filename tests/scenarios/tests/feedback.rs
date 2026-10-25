//! Platform feedback (ADR 0054): anyone posts bugs and suggestions; each person's newest vote
//! counts, and votes can't be cast for someone else.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch};
use identity_integrity::{Feedback, FeedbackKind};
use serde::{Deserialize, Serialize};

mod common;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum VoteDirection {
    Up,
    Down,
}

#[derive(Deserialize, Debug)]
struct FeedbackView {
    id: ActionHash,
    kind: FeedbackKind,
    upvotes: u32,
    downvotes: u32,
    my_vote: Option<VoteDirection>,
}

#[derive(Serialize, Debug)]
struct FeedbackVote {
    id: ActionHash,
    direction: Option<VoteDirection>,
}

#[tokio::test(flavor = "multi_thread")]
async fn feedback_and_votes() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::directory_network(&mut conductors, 1).await;
    let (ann, bob) = (cells[0].zome("identity"), cells[1].zome("identity"));

    let blank: Result<FeedbackView, _> = conductors[0]
        .call_fallible(
            &ann,
            "create_feedback",
            Feedback {
                kind: FeedbackKind::Bug,
                title: " ".into(),
                description: "x".into(),
            },
        )
        .await;
    assert!(blank.is_err(), "feedback needs a title");
    let item: FeedbackView = conductors[0]
        .call(
            &ann,
            "create_feedback",
            Feedback {
                kind: FeedbackKind::Suggestion,
                title: "Dark map tiles".into(),
                description: "The map is bright at night.".into(),
            },
        )
        .await;
    assert_eq!(item.kind, FeedbackKind::Suggestion);
    await_consistency(&cells).await.expect("consistency");

    let listed: Vec<FeedbackView> = conductors[1].call(&bob, "list_feedback", ()).await;
    assert_eq!(listed.len(), 1);
    let vote = |direction| FeedbackVote {
        id: item.id.clone(),
        direction,
    };
    let v: Option<FeedbackView> = conductors[1]
        .call(&bob, "vote_feedback", vote(Some(VoteDirection::Up)))
        .await;
    assert_eq!(
        v.map(|v| (v.upvotes, v.my_vote)),
        Some((1, Some(VoteDirection::Up)))
    );
    await_consistency(&cells).await.expect("consistency");
    let v: Option<FeedbackView> = conductors[1]
        .call(&bob, "vote_feedback", vote(Some(VoteDirection::Down)))
        .await;
    let v = v.expect("item");
    assert_eq!(
        (v.upvotes, v.downvotes),
        (0, 1),
        "the newest vote replaces the old one"
    );
    let _: Option<FeedbackView> = conductors[0]
        .call(&ann, "vote_feedback", vote(Some(VoteDirection::Up)))
        .await;
    await_consistency(&cells).await.expect("consistency");
    let seen: Option<FeedbackView> = conductors[0]
        .call(&ann, "get_feedback", item.id.clone())
        .await;
    let seen = seen.expect("item");
    assert_eq!(
        (seen.upvotes, seen.downvotes, seen.my_vote),
        (1, 1, Some(VoteDirection::Up))
    );

    let v: Option<FeedbackView> = conductors[1].call(&bob, "vote_feedback", vote(None)).await;
    assert_eq!(v.map(|v| (v.downvotes, v.my_vote)), Some((0, None)));
    common::shut_down(conductors).await;
}
