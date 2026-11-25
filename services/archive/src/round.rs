//! One archivist pass (ADR 0031, 0065): every listed public space lives in the archivist's one
//! commons cell, so nothing is joined per project. A space is re-read when it is new to the
//! index or was written to lately (its `recent.<hour>` bucket); every `FULL_EVERY` rounds all
//! are. Any node holding the commons can run it (the seed node now, desktops next).

use std::collections::BTreeSet;

use holo_hash::ActionHashB64;
use holochain_types::prelude::*;

use crate::cards::{
    card, card_activity, help_of, moderation_of, ActivityView, ListingView, ModerationView,
    Profile, ProjectView,
};
use crate::event_time::{event_facts, EventFacts, PlanBoard};
use crate::spaces::{space_of_seed, Scoped};
use crate::{Calls, ProjectCard, ProjectIndex};

/// Every this many rounds (15 s each) every space is re-read: about every 15 minutes.
const FULL_EVERY: u64 = 60;
/// Every this many rounds the recently written spaces are: about once a minute.
const ACTIVE_EVERY: u64 = 4;

/// The archivist's two cells.
#[derive(Clone, Debug)]
pub struct Cells {
    pub directory: CellId,
    pub commons: CellId,
}

/// One pass into `index`. Returns the spaces indexed.
pub async fn archive_round(
    calls: &impl Calls,
    cells: &Cells,
    index: &ProjectIndex,
    round: u64,
) -> anyhow::Result<usize> {
    let listings: Vec<ListingView> = calls
        .call(&cells.directory, "identity", "list_projects", ())
        .await?;
    let full = round.is_multiple_of(FULL_EVERY);
    // Spaces written to this hour or the last; a writer marks a space once an hour, so a
    // marked space is re-read (once a minute) until its hour has passed.
    let active: BTreeSet<ActionHash> = if full || !round.is_multiple_of(ACTIVE_EVERY) {
        BTreeSet::new()
    } else {
        let hour = Timestamp::now().as_micros() / 3_600_000_000;
        let found: Vec<ActionHash> = calls
            .call(
                &cells.commons,
                "project",
                "active_spaces",
                vec![hour - 1, hour],
            )
            .await?;
        found.into_iter().collect()
    };
    let before = index.by_id().await;
    let mut cards = Vec::with_capacity(listings.len());
    for view in listings {
        let Some(space) = space_of_seed(&view.listing.network_seed) else {
            continue; // not a commons space (a listing from before ADR 0065)
        };
        let id = ActionHashB64::from(view.id.clone()).to_string();
        if let Some(card) = before
            .get(&id)
            .filter(|_| !full && !active.contains(&space))
        {
            cards.push(card.clone());
            continue;
        }
        cards.push(Box::pin(read_card(calls, cells, view, space)).await);
    }
    let count = cards.len();
    index.replace(cards).await;
    Ok(count)
}

/// A space's card: its listing, its live counts and its creator's name.
async fn read_card(
    calls: &impl Calls,
    cells: &Cells,
    view: ListingView,
    space: ActionHash,
) -> ProjectCard {
    let commons = &cells.commons;
    let at = |input| Scoped {
        space: space.clone(),
        input,
    };
    let project: Option<ProjectView> = calls
        .call(commons, "project", "get_project", at(()))
        .await
        .unwrap_or(None);
    // The username comes with the card, so guests see it without a session.
    let profile: Option<Profile> = calls
        .call(
            &cells.directory,
            "identity",
            "get_profile",
            view.creator.clone(),
        )
        .await
        .unwrap_or(None);
    // Only the count is used; each message is skipped, not decoded.
    let messages: Vec<serde::de::IgnoredAny> = calls
        .call(commons, "discussion", "list_messages", at(()))
        .await
        .unwrap_or_default();
    let help = project.as_ref().is_some_and(ProjectView::is_help_request);
    // Every space's, for the map's activity markers (ADR 0083); a help request's card
    // shows its one activity too.
    let activities: Vec<ActivityView> = calls
        .call(commons, "activities", "list_activities", at(()))
        .await
        .unwrap_or_default();
    let moderation: Option<ModerationView> = calls
        .call(commons, "discussion", "moderation", at(()))
        .await
        .ok();
    let mut card = card(view, project);
    card.moderation = moderation_of(moderation);
    card.help = help.then(|| help_of(&activities));
    card.activities = activities.iter().map(card_activity).collect();
    if card.mode == "event" {
        event_of(calls, commons, &space).await.apply(&mut card);
    }
    card.creator_name = profile.map(|p| p.name).unwrap_or_default();
    card.comment_count = u32::try_from(messages.len()).unwrap_or(u32::MAX);
    card
}

/// An event's start and place (web's `scheduledAt`): its leading approved plan's.
async fn event_of(calls: &impl Calls, commons: &CellId, space: &ActionHash) -> EventFacts {
    let at = Scoped {
        space: space.clone(),
        input: (),
    };
    let board: Option<PlanBoard> = calls
        .call(commons, "governance", "list_plans", at)
        .await
        .ok();
    board.as_ref().map(event_facts).unwrap_or_default()
}
