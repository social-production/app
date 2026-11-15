//! Comments on platform feedback (Plan, Phase and Feedback Screens §7): flat comments that
//! may answer one another, likes that count once per person, and no comment on a comment
//! from another item.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch};
use identity_integrity::{Feedback, FeedbackKind};
use serde::{Deserialize, Serialize};

mod common;

#[derive(Deserialize, Debug)]
struct FeedbackView {
    id: ActionHash,
    comments: u32,
}

#[derive(Deserialize, Debug)]
struct CommentView {
    id: ActionHash,
    body: String,
    reply_to: Option<ActionHash>,
    likes: u32,
    liked: bool,
}

#[derive(Serialize, Debug)]
struct NewComment {
    feedback: ActionHash,
    body: String,
    reply_to: Option<ActionHash>,
}

#[derive(Serialize, Debug)]
struct Like {
    id: ActionHash,
    like: bool,
}

fn item(title: &str) -> Feedback {
    Feedback {
        kind: FeedbackKind::Bug,
        title: title.into(),
        description: "Long links run off the chat bubble.".into(),
    }
}

fn comment(feedback: &ActionHash, body: &str, reply_to: Option<&ActionHash>) -> NewComment {
    NewComment {
        feedback: feedback.clone(),
        body: body.into(),
        reply_to: reply_to.cloned(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn comments_replies_and_likes() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::directory_network(&mut conductors, 1).await;
    let (ann, bob) = (cells[0].zome("identity"), cells[1].zome("identity"));

    let bug: FeedbackView = conductors[0].call(&ann, "create_feedback", item("Links")).await;
    let other: FeedbackView = conductors[0].call(&ann, "create_feedback", item("Other")).await;
    await_consistency(&cells).await.expect("consistency");

    let first: CommentView = conductors[1]
        .call(&bob, "add_feedback_comment", comment(&bug.id, "Same on Android.", None))
        .await;
    let blank: Result<CommentView, _> = conductors[1]
        .call_fallible(&bob, "add_feedback_comment", comment(&bug.id, "  ", None))
        .await;
    assert!(blank.is_err(), "a comment needs text");
    await_consistency(&cells).await.expect("consistency");

    let reply: CommentView = conductors[0]
        .call(
            &ann,
            "add_feedback_comment",
            comment(&bug.id, "Fixed in the next update.", Some(&first.id)),
        )
        .await;
    assert_eq!(reply.reply_to.as_ref(), Some(&first.id));
    let stray: Result<CommentView, _> = conductors[0]
        .call_fallible(
            &ann,
            "add_feedback_comment",
            comment(&other.id, "Wrong item", Some(&first.id)),
        )
        .await;
    assert!(stray.is_err(), "a reply answers a comment on the same item");

    let liked: Option<CommentView> = conductors[0]
        .call(&ann, "like_feedback_comment", Like { id: first.id.clone(), like: true })
        .await;
    assert_eq!(liked.map(|c| (c.likes, c.liked)), Some((1, true)));
    let again: Option<CommentView> = conductors[0]
        .call(&ann, "like_feedback_comment", Like { id: first.id.clone(), like: true })
        .await;
    assert_eq!(again.map(|c| c.likes), Some(1), "a like counts once per person");
    await_consistency(&cells).await.expect("consistency");

    let listed: Vec<CommentView> = conductors[1]
        .call(&bob, "list_feedback_comments", bug.id.clone())
        .await;
    assert_eq!(
        listed.iter().map(|c| c.body.as_str()).collect::<Vec<_>>(),
        ["Same on Android.", "Fixed in the next update."]
    );
    assert_eq!((listed[0].likes, listed[0].liked), (1, false));
    let counted: Option<FeedbackView> = conductors[1].call(&bob, "get_feedback", bug.id).await;
    assert_eq!(counted.map(|f| f.comments), Some(2));

    let unliked: Option<CommentView> = conductors[0]
        .call(&ann, "like_feedback_comment", Like { id: first.id, like: false })
        .await;
    assert_eq!(unliked.map(|c| (c.likes, c.liked)), Some((0, false)));
}
