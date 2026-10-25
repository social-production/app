//! Follows (ADR 0054): open follows count at once, approval holds them as pending until the
//! person followed decides, and nobody can write follow links for someone else.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch};
use serde::Deserialize;

mod common;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum FollowStatus {
    Accepted,
    Pending,
}

#[derive(Deserialize, Debug)]
struct FollowRow {
    agent: AgentPubKey,
    status: FollowStatus,
}

#[derive(Deserialize, Debug)]
struct FollowView {
    requires_approval: bool,
    followers: Vec<FollowRow>,
    following: Vec<FollowRow>,
    my_status: Option<FollowStatus>,
}

#[tokio::test(flavor = "multi_thread")]
async fn follows_with_and_without_approval() {
    let mut conductors = SweetConductorBatch::standard(3).await;
    let cells = common::directory_network(&mut conductors, 1).await;
    let (ann, bob, cat) = (
        cells[0].zome("identity"),
        cells[1].zome("identity"),
        cells[2].zome("identity"),
    );
    let (ann_key, bob_key, cat_key) = (
        cells[0].agent_pubkey().clone(),
        cells[1].agent_pubkey().clone(),
        cells[2].agent_pubkey().clone(),
    );

    // Open profile: Bob's follow of Ann counts at once.
    let s: Option<FollowStatus> = conductors[1].call(&bob, "follow", ann_key.clone()).await;
    assert_eq!(s, Some(FollowStatus::Accepted));
    let me: Result<Option<FollowStatus>, _> = conductors[1]
        .call_fallible(&bob, "follow", bob_key.clone())
        .await;
    assert!(me.is_err(), "no self-follow");

    // Ann turns approval on: Cat's follow waits, Bob's earlier one still counts.
    let _: bool = conductors[0].call(&ann, "set_follow_approval", true).await;
    await_consistency(&cells).await.expect("consistency");
    let s: Option<FollowStatus> = conductors[2].call(&cat, "follow", ann_key.clone()).await;
    assert_eq!(s, Some(FollowStatus::Pending));
    await_consistency(&cells).await.expect("consistency");

    let requests: Vec<FollowRow> = conductors[0].call(&ann, "follow_requests", ()).await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].agent, cat_key);
    let view: FollowView = conductors[1]
        .call(&bob, "follow_view", ann_key.clone())
        .await;
    assert!(view.requires_approval);
    assert_eq!(view.my_status, Some(FollowStatus::Accepted));
    assert_eq!(view.followers.len(), 2);
    let bobs: FollowView = conductors[0]
        .call(&ann, "follow_view", bob_key.clone())
        .await;
    assert_eq!(bobs.following.len(), 1);
    assert_eq!(bobs.following[0].agent, ann_key);

    // Rejected: the request disappears; Cat can ask again, and Ann accepts this time.
    let _: ActionHash = conductors[0]
        .call(&ann, "reject_follow", cat_key.clone())
        .await;
    await_consistency(&cells).await.expect("consistency");
    let view: FollowView = conductors[2]
        .call(&cat, "follow_view", ann_key.clone())
        .await;
    assert_eq!(view.my_status, None);
    assert_eq!(view.followers.len(), 1);
    let s: Option<FollowStatus> = conductors[2].call(&cat, "follow", ann_key.clone()).await;
    assert_eq!(s, Some(FollowStatus::Pending));
    await_consistency(&cells).await.expect("consistency");
    let _: ActionHash = conductors[0]
        .call(&ann, "accept_follow", cat_key.clone())
        .await;
    await_consistency(&cells).await.expect("consistency");
    let view: FollowView = conductors[2]
        .call(&cat, "follow_view", ann_key.clone())
        .await;
    assert_eq!(view.my_status, Some(FollowStatus::Accepted));
    assert!(view
        .followers
        .iter()
        .all(|r| r.status == FollowStatus::Accepted));

    // Unfollow removes Bob from Ann's followers.
    let (): () = conductors[1].call(&bob, "unfollow", ann_key.clone()).await;
    await_consistency(&cells).await.expect("consistency");
    let view: FollowView = conductors[0]
        .call(&ann, "follow_view", ann_key.clone())
        .await;
    assert_eq!(view.followers.len(), 1);
    assert_eq!(view.followers[0].agent, cat_key);
    common::shut_down(conductors).await;
}

#[derive(serde::Serialize, Debug)]
struct ForgedLink {
    base: AgentPubKey,
    target: AgentPubKey,
}

#[tokio::test(flavor = "multi_thread")]
async fn follow_links_cannot_be_forged() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::directory_network(&mut conductors, 1).await;
    let (ann_key, mal_key) = (
        cells[0].agent_pubkey().clone(),
        cells[1].agent_pubkey().clone(),
    );
    // Mal writes a link claiming Ann follows Mal, using the raw HDK through a test extern.
    let forged: Result<ActionHash, _> = conductors[1]
        .call_fallible(
            &cells[1].zome("identity"),
            "submit_follow_link",
            ForgedLink {
                base: ann_key,
                target: mal_key,
            },
        )
        .await;
    assert!(
        forged.is_err(),
        "a Follows link from someone else's key is invalid"
    );
    common::shut_down(conductors).await;
}
