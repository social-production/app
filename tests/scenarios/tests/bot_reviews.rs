//! Public bot reviews (ADR 0074): three full members' marks open a review, full members vote,
//! and an upheld review pauses the account (its standing reads Paused and its later votes stop
//! counting). New members can't mark, nobody marks themselves, and marks are never deleted.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch, SweetZome};
use serde::{Deserialize, Serialize};

mod common;

#[derive(Serialize, Debug)]
struct BotChoice {
    account: AgentPubKey,
    bot: bool,
}

#[derive(Deserialize, Debug)]
struct BotReviewView {
    status: String,
    bot_votes: u32,
    votes_required: u32,
    my_vote: Option<bool>,
    re_review_from: Option<Timestamp>,
    markers: Vec<AgentPubKey>,
}

#[derive(Deserialize, Debug)]
struct StandingView {
    level: String,
}

#[derive(Deserialize, Debug)]
struct BotNotice {
    account: AgentPubKey,
    kind: String,
}

#[derive(Serialize, Debug)]
struct StandingAt {
    agent: AgentPubKey,
    at: Timestamp,
}

#[tokio::test(flavor = "multi_thread")]
async fn three_marks_open_a_review_that_pauses_a_bot() {
    let mut conductors = SweetConductorBatch::standard(5).await;
    // ann, bob, cat and dan are founders (Full); eve is New.
    let cells = common::directory_network(&mut conductors, 4).await;
    let zome = |i: usize| cells[i].zome("identity");
    let dan = cells[3].agent_pubkey().clone();
    let choice = |bot| BotChoice {
        account: dan.clone(),
        bot,
    };
    let review = |i: usize| {
        let z: SweetZome = zome(i);
        let dan = dan.clone();
        let c = &conductors[i];
        async move { c.call::<_, BotReviewView>(&z, "get_bot_review", dan).await }
    };

    let refused: Result<ActionHash, _> = conductors[4]
        .call_fallible(&zome(4), "set_bot_mark", choice(true))
        .await;
    assert!(refused.is_err(), "a New member can't mark");
    let own: Result<ActionHash, _> = conductors[3]
        .call_fallible(&zome(3), "set_bot_mark", choice(true))
        .await;
    assert!(own.is_err(), "nobody marks themselves");

    for i in 0..3 {
        let _: ActionHash = conductors[i]
            .call(&zome(i), "set_bot_mark", choice(true))
            .await;
    }
    await_consistency(&cells).await.expect("consistency");
    let open = review(4).await;
    assert_eq!(open.status, "open");
    assert_eq!(open.markers.len(), 3);
    assert_eq!(open.votes_required, 3);

    let early: Result<ActionHash, _> = conductors[3]
        .call_fallible(&zome(3), "request_bot_re_review", ())
        .await;
    assert!(early.is_err(), "no re-review while not paused");
    let new_vote: Result<ActionHash, _> = conductors[4]
        .call_fallible(&zome(4), "vote_bot_review", choice(true))
        .await;
    assert!(new_vote.is_err(), "only full members vote");

    let _: ActionHash = conductors[0]
        .call(&zome(0), "vote_bot_review", choice(true))
        .await;
    let mine = review(0).await;
    assert_eq!((mine.my_vote, mine.bot_votes), (Some(true), 1));
    for i in 1..3 {
        let _: ActionHash = conductors[i]
            .call(&zome(i), "vote_bot_review", choice(true))
            .await;
    }
    await_consistency(&cells).await.expect("consistency");

    let upheld = review(4).await;
    assert_eq!(upheld.status, "upheld", "{upheld:?}");
    assert!(upheld.re_review_from.is_some());
    let standing: StandingView = conductors[4]
        .call(&zome(4), "get_standing", dan.clone())
        .await;
    assert_eq!(standing.level, "Paused");
    let now = Timestamp::now();
    let full: bool = conductors[4]
        .call(
            &zome(4),
            "full_standing_before",
            StandingAt {
                agent: dan.clone(),
                at: now,
            },
        )
        .await;
    assert!(!full, "a paused founder's votes no longer count");

    let paused_mark: Result<ActionHash, _> = conductors[3]
        .call_fallible(
            &zome(3),
            "set_bot_mark",
            BotChoice {
                account: cells[4].agent_pubkey().clone(),
                bot: true,
            },
        )
        .await;
    assert!(paused_mark.is_err(), "a paused account is read-only");
    let too_soon: Result<ActionHash, _> = conductors[3]
        .call_fallible(&zome(3), "request_bot_re_review", ())
        .await;
    assert!(too_soon.is_err(), "the re-review opens 30 days later");

    let theirs: Vec<BotNotice> = conductors[3]
        .call(&zome(3), "my_bot_review_notices", ())
        .await;
    let kinds: Vec<&str> = theirs.iter().map(|n| n.kind.as_str()).collect();
    assert_eq!(kinds, ["opened", "upheld"]);
    let markers: Vec<BotNotice> = conductors[0]
        .call(&zome(0), "my_bot_review_notices", ())
        .await;
    assert!(markers.iter().all(|n| n.account == dan));
    assert_eq!(markers.len(), 2);

    common::shut_down(conductors).await;
}
