//! Events for one private community's members (web's `private_community` audience, ADR 0069).
//! The event is a private clone of its own (ADR 0061); the community's clone holds a
//! `ScopeItems` link whose tag is the event's seed, so every member, now or later, finds it.

use hdi::prelude::*;
use sp_event::Audience;

use crate::{charter_at, creator, creator_in_seed, invalid};

/// Longest seed a `ScopeItems` tag may carry: a key, a word and a 24-byte nonce fit easily.
pub const SEED_MAX: usize = 160;

pub fn validate(action: &TypedAction<CreateLinkData>) -> ExternResult<ValidateCallbackResult> {
    if creator()?.is_none() {
        return invalid("only a private community lists events for its members");
    }
    let community = match action.base_address.clone().into_action_hash() {
        Some(hash) => charter_at(&hash)?,
        None => None,
    };
    if community.is_none_or(|(c, _)| c.mode != sp_scope::COMMUNITY) {
        return invalid("events are listed on a community");
    }
    let tag = action.tag.0.as_slice();
    let Ok(seed) = std::str::from_utf8(tag) else {
        return invalid("a listed event's tag is its seed");
    };
    if seed.len() > SEED_MAX || !Audience::of_seed(seed).is_private() {
        return invalid("only private events are listed on a private community");
    }
    if creator_in_seed(seed).as_ref() != Some(action.author()) {
        return invalid("only an event's creator lists it on a community");
    }
    Ok(ValidateCallbackResult::Valid)
}
