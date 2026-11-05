//! Public bot reviews (ADR 0074): three full members' marks open a review, full members vote,
//! and an upheld review pauses the account (its standing reads Paused and its later votes stop
//! counting). New members can't mark, nobody marks themselves, and marks are never deleted.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetCell, SweetConductorBatch};
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

/// Five directory agents: ann, bob, cat and dan are founders (Full); eve is New. Dan is the
/// account under review.
struct Net {
    conductors: SweetConductorBatch,
    cells: Vec<SweetCell>,
}

impl Net {
    fn dan(&self) -> AgentPubKey {
        self.cells[3].agent_pubkey().clone()
    }

    fn choice(&self, bot: bool) -> BotChoice {
        BotChoice {
            account: self.dan(),
            bot,
        }
    }

    async fn call<
        I: Serialize + std::fmt::Debug,
        O: serde::de::DeserializeOwned + std::fmt::Debug,
    >(
        &self,
        i: usize,
        func: &str,
        payload: I,
    ) -> O {
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

    async fn review(&self, i: usize) -> BotReviewView {
        self.call(i, "get_bot_review", self.dan()).await
    }

    async fn settle(&self) {
        await_consistency(&self.cells).await.expect("consistency");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn three_marks_open_a_review_that_pauses_a_bot() {
    let mut conductors = SweetConductorBatch::standard(5).await;
    let cells = common::directory_network(&mut conductors, 4).await;
    let net = Net { conductors, cells };
    marks_open_the_review(&net).await;
    votes_uphold_it(&net).await;
    the_paused_account_is_read_only(&net).await;
    notices_follow_the_review(&net).await;
    common::shut_down(net.conductors).await;
}

async fn marks_open_the_review(net: &Net) {
    assert!(
        net.refused(4, "set_bot_mark", net.choice(true)).await,
        "a New member can't mark"
    );
    assert!(
        net.refused(3, "set_bot_mark", net.choice(true)).await,
        "nobody marks themselves"
    );
    for i in 0..3 {
        let _: ActionHash = net.call(i, "set_bot_mark", net.choice(true)).await;
    }
    net.settle().await;
    let open = net.review(4).await;
    assert_eq!(open.status, "open");
    assert_eq!(open.markers.len(), 3);
    assert_eq!(open.votes_required, 3);
    assert!(
        net.refused(3, "request_bot_re_review", ()).await,
        "no re-review while not paused"
    );
    assert!(
        net.refused(4, "vote_bot_review", net.choice(true)).await,
        "only full members vote"
    );
}

async fn votes_uphold_it(net: &Net) {
    let _: ActionHash = net.call(0, "vote_bot_review", net.choice(true)).await;
    let mine = net.review(0).await;
    assert_eq!((mine.my_vote, mine.bot_votes), (Some(true), 1));
    for i in 1..3 {
        let _: ActionHash = net.call(i, "vote_bot_review", net.choice(true)).await;
    }
    net.settle().await;
    let upheld = net.review(4).await;
    assert_eq!(upheld.status, "upheld", "{upheld:?}");
    assert!(upheld.re_review_from.is_some());
}

async fn the_paused_account_is_read_only(net: &Net) {
    let standing: StandingView = net.call(4, "get_standing", net.dan()).await;
    assert_eq!(standing.level, "Paused");
    let at = StandingAt {
        agent: net.dan(),
        at: Timestamp::now(),
    };
    let full: bool = net.call(4, "full_standing_before", at).await;
    assert!(!full, "a paused founder's votes no longer count");
    let eve = BotChoice {
        account: net.cells[4].agent_pubkey().clone(),
        bot: true,
    };
    assert!(
        net.refused(3, "set_bot_mark", eve).await,
        "a paused account is read-only"
    );
    assert!(
        net.refused(3, "request_bot_re_review", ()).await,
        "the re-review opens 30 days later"
    );
}

async fn notices_follow_the_review(net: &Net) {
    let theirs: Vec<BotNotice> = net.call(3, "my_bot_review_notices", ()).await;
    let kinds: Vec<&str> = theirs.iter().map(|n| n.kind.as_str()).collect();
    assert_eq!(kinds, ["opened", "upheld"]);
    let markers: Vec<BotNotice> = net.call(0, "my_bot_review_notices", ()).await;
    assert!(markers.iter().all(|n| n.account == net.dan()));
    assert_eq!(markers.len(), 2);
}
