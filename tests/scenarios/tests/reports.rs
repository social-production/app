//! Reports (ADR 0071): web's ⋮ Report on a space. One open report per item; the reporter's
//! yes comes with it; only Full members vote, never the item's author; three yes votes in a
//! small audience remove a spam report's item for every reader.

use holochain::prelude::*;
use serde::{Deserialize, Serialize};

mod common;

use common::net::Net;

#[derive(Serialize, Debug)]
struct ReportInput {
    reason: String,
    description: String,
}

#[derive(Serialize, Debug)]
struct ReportVoteInput {
    report: ActionHash,
    yes: bool,
}

#[derive(Deserialize, Debug)]
struct ReportView {
    id: ActionHash,
    resolution: String,
    yes: u32,
    no: u32,
    mine: Option<bool>,
    audience: u32,
    delete_quorum: u32,
}

#[derive(Deserialize, Debug)]
struct ModerationView {
    state: String,
    report: Option<ReportView>,
}

fn spam(text: &str) -> ReportInput {
    ReportInput {
        reason: "spam".into(),
        description: text.into(),
    }
}

async fn moderation(net: &Net, i: usize) -> ModerationView {
    net.call(i, "discussion", "moderation", ()).await
}

async fn vote(
    net: &Net,
    i: usize,
    report: &ActionHash,
    yes: bool,
) -> Result<ModerationView, String> {
    net.try_call(
        i,
        "discussion",
        "vote_report",
        ReportVoteInput {
            report: report.clone(),
            yes,
        },
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn three_yes_votes_remove_a_spam_report_s_item() {
    // Agents 0–3 are Full; 4 is new. Agent 0 created the project; everyone joined.
    let net = Net::new(5, 4, "reports").await;
    assert_eq!(moderation(&net, 4).await.state, "visible");

    let own: Result<ModerationView, String> = net
        .try_call(0, "discussion", "report_space", spam("mine"))
        .await;
    assert!(own.is_err(), "you can't report yourself");

    let after: ModerationView = net
        .call(1, "discussion", "report_space", spam("Adverts for pills"))
        .await;
    assert_eq!(after.state, "under_review");
    let report = after.report.expect("the open report");
    assert_eq!((report.yes, report.no, report.mine), (1, 0, Some(true)));
    // Five members: an audience of four (the author left out), so three votes delete.
    assert_eq!((report.audience, report.delete_quorum), (4, 3));
    net.settle().await;

    let second: Result<ModerationView, String> = net
        .try_call(2, "discussion", "report_space", spam("Again"))
        .await;
    assert!(second.is_err(), "one open report per item");
    assert!(
        vote(&net, 4, &report.id, true).await.is_err(),
        "only Full members vote"
    );
    assert!(
        vote(&net, 0, &report.id, false).await.is_err(),
        "not on your own item"
    );

    let two = vote(&net, 2, &report.id, true)
        .await
        .expect("a Full member's vote");
    assert_eq!(two.state, "under_review");
    net.settle().await;
    let three = vote(&net, 3, &report.id, true)
        .await
        .expect("the deciding vote");
    assert_eq!(three.state, "removed");
    net.settle().await;

    let seen = moderation(&net, 4).await;
    assert_eq!(seen.state, "removed", "every reader replays the same votes");
    let report = seen.report.expect("the removing report");
    assert_eq!(
        (report.resolution.as_str(), report.yes, report.no),
        ("removed", 3, 0)
    );
    assert!(
        vote(&net, 1, &report.id, false).await.is_err(),
        "resolved reports take no votes"
    );
    net.shut_down().await;
}
