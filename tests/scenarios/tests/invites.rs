//! Event invitations on the event itself (ADR 0111): sending an inbox invite also writes an
//! `Invite` record, so every organizer lists who is invited, by whom and when. In a private
//! event only organizers record invites; anyone may record one on a public event.

use holochain::prelude::*;
use serde::Deserialize;

mod common;

use common::net::{charter, Net};

#[derive(Deserialize, Debug)]
struct InviteView {
    agent: AgentPubKey,
    by: AgentPubKey,
    #[allow(dead_code)]
    at: Timestamp,
}

async fn invites(net: &Net, i: usize) -> Vec<InviteView> {
    net.call(i, "project", "event_invites", ()).await
}

/// Agent 0 creates; agent 1 is a co-organizer; agent 2 is a member; agent 3 is invited.
#[tokio::test(flavor = "multi_thread")]
async fn organizers_see_every_invite_to_a_private_event() {
    let net = Net::with_charter(4, 2, "private.picnic", charter("event")).await;
    let key = |i: usize| net.cells[i].1.agent_pubkey().clone();
    let _: ActionHash = net.call(0, "project", "add_organizer", key(1)).await;
    net.settle().await;

    let refused = net.try_call::<_, ()>(2, "project", "record_invite", key(3)).await;
    assert!(refused.is_err(), "only organizers invite to a private event");
    let _: () = net.call(1, "project", "record_invite", key(3)).await;
    let _: () = net.call(1, "project", "record_invite", key(3)).await;
    let _: () = net.call(0, "project", "record_invite", key(2)).await;
    net.settle().await;

    let seen = invites(&net, 0).await;
    assert_eq!(seen.len(), 2, "one row per person: {seen:?}");
    assert!(seen.iter().any(|i| i.agent == key(3) && i.by == key(1)));
    assert!(seen.iter().any(|i| i.agent == key(2) && i.by == key(0)));
    net.shut_down().await;
}

/// A removed organizer's later invites don't count (ADR 0073).
#[tokio::test(flavor = "multi_thread")]
async fn a_removed_organizers_later_invites_are_left_out() {
    let net = Net::with_charter(3, 2, "private.lunch", charter("event")).await;
    let key = |i: usize| net.cells[i].1.agent_pubkey().clone();
    let _: ActionHash = net.call(0, "project", "add_organizer", key(1)).await;
    net.settle().await;
    let _: () = net.call(0, "project", "remove_organizer", key(1)).await;
    net.settle().await;
    let late = net.try_call::<_, ()>(1, "project", "record_invite", key(2)).await;
    assert!(late.is_err(), "a removed organizer can't record invites");
    assert!(invites(&net, 0).await.is_empty());
    net.shut_down().await;
}

/// Anyone may record inviting someone to a public event; you can't invite yourself.
#[tokio::test(flavor = "multi_thread")]
async fn anyone_records_an_invite_to_a_public_event() {
    let net = Net::with_charter(3, 2, "fair", charter("event")).await;
    let key = |i: usize| net.cells[i].1.agent_pubkey().clone();
    let _: () = net.call(1, "project", "record_invite", key(2)).await;
    let _: () = net.call(1, "project", "record_invite", key(1)).await;
    net.settle().await;
    let seen = invites(&net, 0).await;
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert_eq!((&seen[0].agent, &seen[0].by), (&key(2), &key(1)));
    net.shut_down().await;
}
