//! M4b discussion threads (ADR 0041): anyone in the project's network starts a thread,
//! comments and replies; up/down votes count once per person; lists sort like web's feeds.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch, SweetZome};
use serde::{Deserialize, Serialize};

mod common;

/// Mirrors `discussion::ThreadInput` (integrity crates can't share one test binary).
#[derive(Serialize, Debug)]
struct ThreadInput {
    title: String,
    body: String,
}

#[derive(Serialize, Debug)]
struct ThreadQuery {
    sort: String,
    window: String,
    utc_offset_minutes: i32,
}

#[derive(Serialize, Debug)]
struct CommentInput {
    thread: ActionHash,
    parent: Option<ActionHash>,
    body: String,
}

#[derive(Serialize, Debug)]
struct VoteInput {
    target: ActionHash,
    direction: i8,
}

#[derive(Deserialize, Debug)]
struct ThreadView {
    id: ActionHash,
    author: AgentPubKey,
    title: String,
    body: String,
    vote_count: i32,
    my_vote: i8,
    comment_count: u32,
}

#[derive(Deserialize, Debug)]
struct ThreadDetail {
    thread: ThreadView,
    comments: Vec<CommentView>,
}

#[derive(Deserialize, Debug)]
struct CommentView {
    id: ActionHash,
    author: AgentPubKey,
    parent: Option<ActionHash>,
    body: String,
    vote_count: i32,
    my_vote: i8,
}

fn thread(title: &str, body: &str) -> ThreadInput {
    ThreadInput {
        title: title.into(),
        body: body.into(),
    }
}

fn sorted(sort: &str) -> ThreadQuery {
    ThreadQuery {
        sort: sort.into(),
        window: "all".into(),
        utc_offset_minutes: 0,
    }
}

fn comment(thread: &ActionHash, parent: Option<&ActionHash>, body: &str) -> CommentInput {
    CommentInput {
        thread: thread.clone(),
        parent: parent.cloned(),
        body: body.into(),
    }
}

fn vote(target: &ActionHash, direction: i8) -> VoteInput {
    VoteInput {
        target: target.clone(),
        direction,
    }
}

async fn detail(c: &SweetConductorBatch, i: usize, z: &SweetZome, id: &ActionHash) -> ThreadDetail {
    c[i].call(z, "get_thread", id.clone()).await
}

#[tokio::test(flavor = "multi_thread")]
async fn anyone_starts_threads_and_comments() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::project_network(&mut conductors, "threads-read").await;
    let (ann, bob) = (cells[0].zome("discussion"), cells[1].zome("discussion"));

    // Bob never joined: as on web, anyone signed in may start a thread.
    let id: ActionHash = conductors[1]
        .call(
            &bob,
            "create_thread",
            thread("  Soil test?  ", " Who has a kit? "),
        )
        .await;
    let mine: Vec<ThreadView> = conductors[1]
        .call(&bob, "list_threads", sorted("recent"))
        .await;
    assert_eq!(mine.len(), 1, "your own thread shows before gossip");
    assert_eq!(
        (mine[0].title.as_str(), mine[0].body.as_str()),
        ("Soil test?", "Who has a kit?"),
        "title and opening post are trimmed"
    );

    await_consistency(&cells).await.expect("consistency");
    let first: ActionHash = conductors[0]
        .call(&ann, "add_comment", comment(&id, None, "I do"))
        .await;
    let own = detail(&conductors, 0, &ann, &id).await;
    assert_eq!(own.comments.len(), 1, "your own comment shows at once");
    await_consistency(&cells).await.expect("consistency");
    let reply: ActionHash = conductors[1]
        .call(
            &bob,
            "add_comment",
            comment(&id, Some(&first), "Can I borrow it?"),
        )
        .await;
    await_consistency(&cells).await.expect("consistency");

    let seen = detail(&conductors, 0, &ann, &id).await;
    assert_eq!(&seen.thread.author, cells[1].agent_pubkey());
    let ids: Vec<&ActionHash> = seen.comments.iter().map(|c| &c.id).collect();
    assert_eq!(ids, vec![&first, &reply], "oldest first");
    assert_eq!(&seen.comments[0].author, cells[0].agent_pubkey());
    assert_eq!(seen.comments[1].parent.as_ref(), Some(&first));
    assert_eq!(seen.comments[1].body, "Can I borrow it?");
    let listed: Vec<ThreadView> = conductors[0].call(&ann, "list_threads", sorted("")).await;
    assert_eq!(listed[0].id, id);
    assert_eq!(
        listed[0].comment_count, 2,
        "cards count comments and replies"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_threads_and_comments_are_rejected() {
    let mut conductors = SweetConductorBatch::standard(1).await;
    let cells = common::project_network(&mut conductors, "threads-bad").await;
    let ann = cells[0].zome("discussion");
    let c = &conductors[0];

    for (input, why) in [
        (thread("  ", "Body"), "a thread needs a title"),
        (
            thread(&"t".repeat(201), "Body"),
            "a title is at most 200 characters",
        ),
        (thread("Title", " "), "a thread needs an opening post"),
    ] {
        let r: Result<ActionHash, _> = c.call_fallible(&ann, "create_thread", input).await;
        assert!(r.is_err(), "{why}");
    }

    let one: ActionHash = c.call(&ann, "create_thread", thread("One", "First")).await;
    let two: ActionHash = c.call(&ann, "create_thread", thread("Two", "Second")).await;
    let in_one: ActionHash = c.call(&ann, "add_comment", comment(&one, None, "Hi")).await;
    for (input, why) in [
        (comment(&one, None, "   "), "a comment needs text"),
        (
            comment(&one, None, &"x".repeat(4_001)),
            "a comment is at most 4,000 characters",
        ),
        (
            comment(&in_one, None, "Hi"),
            "a comment belongs to a thread",
        ),
        (
            comment(&two, Some(&in_one), "Hi"),
            "a reply stays in its thread",
        ),
    ] {
        let r: Result<ActionHash, _> = c.call_fallible(&ann, "add_comment", input).await;
        assert!(r.is_err(), "{why}");
    }
    let odd: Result<(), _> = c.call_fallible(&ann, "cast_vote", vote(&one, 5)).await;
    assert!(odd.is_err(), "a vote is up, down or neutral");
}

#[tokio::test(flavor = "multi_thread")]
async fn votes_count_once_per_person() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::project_network(&mut conductors, "threads-votes").await;
    let (ann, bob) = (cells[0].zome("discussion"), cells[1].zome("discussion"));

    let quiet: ActionHash = conductors[0]
        .call(&ann, "create_thread", thread("Quiet", "Nobody votes"))
        .await;
    let loud: ActionHash = conductors[0]
        .call(&ann, "create_thread", thread("Loud", "Everyone votes"))
        .await;
    let (): () = conductors[0].call(&ann, "cast_vote", vote(&loud, 1)).await;
    let (): () = conductors[0].call(&ann, "cast_vote", vote(&loud, 1)).await;
    let own = detail(&conductors, 0, &ann, &loud).await;
    assert_eq!(
        (own.thread.vote_count, own.thread.my_vote),
        (1, 1),
        "voting twice counts once"
    );

    await_consistency(&cells).await.expect("consistency");
    let (): () = conductors[1].call(&bob, "cast_vote", vote(&loud, 1)).await;
    let (): () = conductors[1]
        .call(&bob, "cast_vote", vote(&quiet, -1))
        .await;
    await_consistency(&cells).await.expect("consistency");
    let top: Vec<ThreadView> = conductors[0]
        .call(&ann, "list_threads", sorted("top"))
        .await;
    let order: Vec<&ActionHash> = top.iter().map(|t| &t.id).collect();
    assert_eq!(order, vec![&loud, &quiet], "top sorts by score");
    assert_eq!((top[0].vote_count, top[1].vote_count), (2, -1));
    let oldest: Vec<ThreadView> = conductors[0]
        .call(&ann, "list_threads", sorted("oldest"))
        .await;
    assert_eq!(oldest[0].id, quiet);

    // Changing your mind replaces your vote; neutral takes it back.
    let (): () = conductors[1].call(&bob, "cast_vote", vote(&loud, -1)).await;
    let mine = detail(&conductors, 1, &bob, &loud).await;
    assert_eq!((mine.thread.vote_count, mine.thread.my_vote), (0, -1));
    let (): () = conductors[1].call(&bob, "cast_vote", vote(&loud, 0)).await;
    await_consistency(&cells).await.expect("consistency");
    let seen = detail(&conductors, 0, &ann, &loud).await;
    assert_eq!(seen.thread.vote_count, 1, "only Ann's vote is left");

    // Comments take votes too.
    let note: ActionHash = conductors[0]
        .call(&ann, "add_comment", comment(&loud, None, "Thanks"))
        .await;
    let (): () = conductors[0].call(&ann, "cast_vote", vote(&note, -1)).await;
    let voted = detail(&conductors, 0, &ann, &loud).await;
    assert_eq!(
        (voted.comments[0].vote_count, voted.comments[0].my_vote),
        (-1, -1)
    );
}
