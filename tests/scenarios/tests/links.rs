//! Links between projects (web's Links tab, ADR 0042): a member of one project proposes a
//! link and mirrors it into the other; each project votes on its own half, and the link is
//! active once both pass. Severing works the same way.

use holochain::prelude::ActionHash;
use sp_governance::links::SideStatus;

mod common;

use common::links::{link, Pair};

#[tokio::test(flavor = "multi_thread")]
async fn both_projects_approve_a_link_and_either_can_sever_it() {
    let pair = Pair::new().await;
    pair.refused(1, 0, link(&pair.refs[1], "Why"), "only members propose")
        .await;
    pair.refused(0, 0, link(&pair.refs[0], "Why"), "not to itself")
        .await;
    pair.refused(0, 0, link(&pair.refs[1], "  "), "a reason is required")
        .await;

    let id = pair.request(0, 0, [None, None]).await;
    let a = pair.links(0).await;
    let open = &a.pending[0];
    assert_eq!(
        open.mine.request.relationship, "Linked",
        "web's default label"
    );
    assert_eq!(open.mine.tally.yes, 0, "no proposer vote, as on web");
    let mirror = open.theirs.clone().expect("B received its half");
    assert_eq!(mirror.request.source.as_ref(), Some(&id));

    pair.vote(0, 0, &id, "Yes").await;
    let a = pair.links(0).await;
    assert_eq!(a.pending[0].mine.status, SideStatus::Approved);
    assert_eq!(
        a.pending[0].status,
        SideStatus::Open,
        "B hasn't approved yet"
    );

    pair.vote(1, 1, &mirror.id, "Yes").await;
    let (a, b) = (pair.links(0).await, pair.links(1).await);
    assert!(a.pending.is_empty() && b.pending.is_empty());
    assert_eq!(
        (a.active.len(), b.active.len()),
        (1, 1),
        "both sides see the link"
    );
    let creates = [
        Some(a.active[0].create.mine.id.clone()),
        Some(b.active[0].create.mine.id.clone()),
    ];
    sever_both_ways(&pair, creates).await;
}

/// B proposes a sever and A keeps the link; then A proposes one and both pass it.
async fn sever_both_ways(pair: &Pair, creates: [Option<ActionHash>; 2]) {
    let sever = pair.request(1, 1, creates.clone()).await;
    let b = pair.links(1).await;
    let open = b.active[0]
        .sever
        .as_ref()
        .expect("an open sever rides on the link");
    let a_half = open
        .theirs
        .as_ref()
        .expect("A received its half")
        .id
        .clone();
    pair.vote(1, 1, &sever, "Yes").await;
    pair.vote(0, 0, &a_half, "No").await;
    let a = pair.links(0).await;
    assert_eq!(a.active.len(), 1, "a rejected sever keeps the link");
    assert_eq!(a.past[0].status, SideStatus::Rejected);

    // A second sever passes on both sides.
    let sever = pair.request(0, 0, creates).await;
    let b_half = pair.links(0).await.active[0]
        .sever
        .as_ref()
        .and_then(|s| s.theirs.clone())
        .expect("B received its half of the second sever")
        .id;
    pair.vote(0, 0, &sever, "Yes").await;
    pair.vote(1, 1, &b_half, "Yes").await;
    let (a, b) = (pair.links(0).await, pair.links(1).await);
    assert!(a.active.is_empty() && b.active.is_empty());
    assert_eq!((a.historical.len(), b.historical.len()), (1, 1));
    assert_eq!(a.past.len(), 2, "both sever votes are past votes");
}
