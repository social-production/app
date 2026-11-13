//! Platform moderators, the weekly active count and deactivation (ADRs 0095, 0088). Full
//! members volunteer and vote; the replay seats and removes moderators; one `Active` link per
//! person per week; a deactivated account's chain is closed.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetCell, SweetConductorBatch};
use serde::{Deserialize, Serialize};

mod common;

const WEEK_US: i64 = 7 * 24 * 60 * 60 * 1_000_000;

#[derive(Serialize, Debug)]
struct BoardVote {
    account: AgentPubKey,
    vote: String,
}

#[derive(Deserialize, Debug)]
struct BoardRow {
    agent: AgentPubKey,
    standing: String,
    yes: u32,
    no: u32,
    my_vote: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct BoardView {
    weekly_active: u32,
    quorum: u32,
    rows: Vec<BoardRow>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct WeeklyActive {
    week: i64,
    count: u32,
}

/// ann, bob, cat and dan are founders (Full); eve is New. Ann stands for the board.
struct Net {
    conductors: SweetConductorBatch,
    cells: Vec<SweetCell>,
}

impl Net {
    fn agent(&self, i: usize) -> AgentPubKey {
        self.cells[i].agent_pubkey().clone()
    }

    async fn call<I, O>(&self, i: usize, func: &str, payload: I) -> O
    where
        I: Serialize + std::fmt::Debug,
        O: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let zome = self.cells[i].zome("identity");
        self.conductors[i].call(&zome, func, payload).await
    }

    async fn refused<I: Serialize + std::fmt::Debug>(
        &self,
        i: usize,
        func: &str,
        payload: I,
    ) -> bool {
        let zome = self.cells[i].zome("identity");
        let r: Result<ActionHash, _> = self.conductors[i].call_fallible(&zome, func, payload).await;
        r.is_err()
    }

    async fn vote(&self, i: usize, vote: &str) -> ActionHash {
        let v = BoardVote {
            account: self.agent(0),
            vote: vote.into(),
        };
        self.call(i, "vote_board", v).await
    }

    async fn board(&self, i: usize) -> BoardView {
        self.call(i, "get_board", ()).await
    }

    async fn settle(&self) {
        await_consistency(&self.cells).await.expect("consistency");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn moderators_active_weeks_and_deactivation() {
    let mut conductors = SweetConductorBatch::standard(5).await;
    let cells = common::directory_network(&mut conductors, 4).await;
    let net = Net { conductors, cells };
    volunteers_are_seated_and_removed(&net).await;
    one_active_link_a_week(&net).await;
    a_deactivated_chain_is_closed(&net).await;
    common::shut_down(net.conductors).await;
}

async fn volunteers_are_seated_and_removed(net: &Net) {
    assert!(
        net.refused(4, "volunteer_board", true).await,
        "a New member can't volunteer"
    );
    let _: ActionHash = net.call(0, "volunteer_board", true).await;
    assert!(
        net.refused(
            0,
            "vote_board",
            BoardVote {
                account: net.agent(0),
                vote: "yes".into()
            }
        )
        .await
    );
    assert!(
        net.refused(
            4,
            "vote_board",
            BoardVote {
                account: net.agent(0),
                vote: "yes".into()
            }
        )
        .await
    );
    net.settle().await;
    // Nobody was active last week, so the quorum is 0 and a volunteer is seated at once (web).
    let board = net.board(4).await;
    assert_eq!((board.weekly_active, board.quorum), (0, 0));
    assert_eq!(board.rows.len(), 1);
    assert_eq!(
        (board.rows[0].agent.clone(), board.rows[0].standing.as_str()),
        (net.agent(0), "moderator")
    );
    let moderates: bool = net.call(4, "is_moderator", net.agent(0)).await;
    assert!(moderates);
    // One no vote: approval under 66% removes a moderator at once.
    net.vote(1, "no").await;
    net.settle().await;
    assert!(net.board(4).await.rows.is_empty(), "removed");
    // Volunteering again makes a candidate; two yes votes reach 66% and seat it.
    let _: ActionHash = net.call(0, "volunteer_board", true).await;
    net.settle().await;
    assert_eq!(net.board(4).await.rows[0].standing, "candidate");
    net.vote(2, "yes").await;
    net.vote(3, "yes").await;
    net.settle().await;
    let board = net.board(2).await;
    let row = &board.rows[0];
    assert_eq!(
        (row.standing.as_str(), row.yes, row.no, row.my_vote),
        ("moderator", 2, 1, Some(true))
    );
    // A neutral vote withdraws bob's no; withdrawing the candidacy leaves the board.
    net.vote(1, "neutral").await;
    let _: ActionHash = net.call(0, "volunteer_board", false).await;
    net.settle().await;
    assert!(net.board(4).await.rows.is_empty(), "withdrawn");
}

async fn one_active_link_a_week(net: &Net) {
    // Volunteering and voting already wrote ann's to dan's link for this week.
    let wrote: bool = net.call(0, "record_active", ()).await;
    assert!(!wrote, "one a week");
    let eve: bool = net.call(4, "record_active", ()).await;
    assert!(eve);
    let week: WeeklyActive = net.call(0, "weekly_active", None::<Timestamp>).await;
    // `week` is last week (the one counted); this week is `week + 1`.
    assert!(
        net.refused(0, "submit_active_link", week.week + 1).await,
        "a second link this week"
    );
    assert!(
        net.refused(0, "submit_active_link", week.week + 2).await,
        "next week's path"
    );
    assert!(
        net.refused(1, "submit_active_link", week.week).await,
        "last week's path"
    );
    net.settle().await;
    assert_eq!(week.count, 0, "the current week isn't counted yet");
    // Counted from next week: the four founders, not eve (New).
    let later = Timestamp::from_micros(Timestamp::now().as_micros() + WEEK_US);
    let next: WeeklyActive = net.call(4, "weekly_active", Some(later)).await;
    assert_eq!(
        next,
        WeeklyActive {
            week: week.week + 1,
            count: 4
        }
    );
}

async fn a_deactivated_chain_is_closed(net: &Net) {
    let none: Option<Timestamp> = net.call(0, "deactivated_since", net.agent(3)).await;
    assert!(none.is_none());
    let _: Timestamp = net.call(3, "deactivate_account", ()).await;
    let own: Option<Timestamp> = net.call(3, "deactivated_since", net.agent(3)).await;
    assert!(own.is_some());
    assert!(
        net.refused(3, "volunteer_board", true).await,
        "nothing is written after it"
    );
    net.settle().await;
    let seen: Option<Timestamp> = net.call(0, "deactivated_since", net.agent(3)).await;
    assert!(seen.is_some());
}
