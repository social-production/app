//! Notices (ADR 0108): a writer's app tells the people a new card is for, at once, with a
//! remote signal through the directory's `notices` zome. Only someone who enabled notices
//! hears them, the writer is never told about their own write, and a notice whose link leaves
//! the app is refused.

use std::time::Duration;

use holochain::prelude::*;
use holochain::sweettest::SweetConductorBatch;
use serde::{Deserialize, Serialize};

mod common;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Notice {
    kind: String,
    category: String,
    title: String,
    body: String,
    link: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Envelope {
    notice: Notice,
}

#[derive(Serialize, Debug)]
struct SendInput {
    to: Vec<AgentPubKey>,
    notice: Notice,
}

fn reply() -> Notice {
    Notice {
        kind: "reply".into(),
        category: "comments".into(),
        title: "Garden build".into(),
        body: "Replied to your comment: count me in".into(),
        link: "/projects/abc?tab=chat".into(),
    }
}

/// The next notice `signals` hears within `wait`, if any.
async fn heard(
    signals: &mut tokio::sync::broadcast::Receiver<Signal>,
    wait: Duration,
) -> Option<Notice> {
    let deadline = tokio::time::Instant::now() + wait;
    loop {
        let left = deadline.checked_duration_since(tokio::time::Instant::now())?;
        let Ok(Ok(signal)) = tokio::time::timeout(left, signals.recv()).await else {
            return None;
        };
        if let Signal::App { signal, .. } = signal {
            if let Ok(envelope) = signal.into_inner().decode::<Envelope>() {
                return Some(envelope.notice);
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_notice_reaches_whoever_enabled_them() {
    let mut conductors = SweetConductorBatch::standard(3).await;
    let cells = common::directory_network(&mut conductors, 3).await;
    let (ann, ben, cat) = (&cells[0], &cells[1], &cells[2]);
    let first: bool = conductors[0]
        .call(&ann.zome("notices"), "enable_notices", ())
        .await;
    let again: bool = conductors[0]
        .call(&ann.zome("notices"), "enable_notices", ())
        .await;
    assert!(first && !again, "the grant is made once");

    let mut ann_hears = conductors[0].raw_handle().app_signals("sp".into());
    let mut ben_hears = conductors[1].raw_handle().app_signals("sp".into());
    let mut cat_hears = conductors[2].raw_handle().app_signals("sp".into());

    // Ben replies; the notice goes to Ann (enabled), Cat (not enabled) and Ben himself.
    let to = vec![
        ann.agent_pubkey().clone(),
        cat.agent_pubkey().clone(),
        ben.agent_pubkey().clone(),
    ];
    let sent: usize = conductors[1]
        .call(
            &ben.zome("notices"),
            "send_notice",
            SendInput {
                to,
                notice: reply(),
            },
        )
        .await;
    assert_eq!(sent, 2, "the writer is left out");

    let got = heard(&mut ann_hears, Duration::from_mins(1)).await;
    assert_eq!(got, Some(reply()), "Ann hears the notice as it was sent");
    let short = Duration::from_secs(5);
    assert_eq!(
        heard(&mut cat_hears, short).await,
        None,
        "no grant, no notice"
    );
    assert_eq!(
        heard(&mut ben_hears, short).await,
        None,
        "the writer hears nothing"
    );

    let outside = Notice {
        link: "https://example.com".into(),
        ..reply()
    };
    let refused: Result<usize, _> = conductors[1]
        .call_fallible(
            &ben.zome("notices"),
            "send_notice",
            SendInput {
                to: vec![ann.agent_pubkey().clone()],
                notice: outside,
            },
        )
        .await;
    assert!(refused.is_err(), "a link outside the app is refused");

    common::shut_down(conductors).await;
}
