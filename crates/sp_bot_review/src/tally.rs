use std::collections::{BTreeMap, BTreeSet};

use sp_governance::quorum::required_votes;

use crate::{BOT_SHARE_PERCENT, MIN_VOTES};

/// web's `_tally`, with the population sized from the people involved (ADR 0074).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tally {
    pub bot_votes: u32,
    pub real_votes: u32,
    pub votes_required: u32,
    pub passing: bool,
    pub can_still_pass: bool,
}

fn share_ok(bot: u32, total: u32) -> bool {
    total > 0 && bot * 100 >= BOT_SHARE_PERCENT * total
}

/// `votes`: each counted voter's latest vote (true = bot). `markers`: the counted markers.
/// web sizes the population from the site's weekly actives; a P2P network has no global
/// count, so it is everyone who marked or voted, and never fewer than `MIN_VOTES` decide.
#[must_use]
pub fn tally<A: Ord>(votes: &BTreeMap<A, bool>, markers: &BTreeSet<A>) -> Tally {
    let bot_votes = u32::try_from(votes.values().filter(|bot| **bot).count()).unwrap_or(u32::MAX);
    let total = u32::try_from(votes.len()).unwrap_or(u32::MAX);
    let real_votes = total - bot_votes;
    let people: BTreeSet<&A> = votes.keys().chain(markers.iter()).collect();
    let population = u32::try_from(people.len()).unwrap_or(u32::MAX);
    let needed = MIN_VOTES.max(required_votes(population));
    let remaining = population.max(needed).saturating_sub(total);
    let passing = total >= needed && share_ok(bot_votes, total);
    let max_total = total + remaining;
    let can_still_pass =
        !passing && max_total >= needed && share_ok(bot_votes + remaining, max_total);
    Tally {
        bot_votes,
        real_votes,
        votes_required: needed,
        passing,
        can_still_pass,
    }
}
