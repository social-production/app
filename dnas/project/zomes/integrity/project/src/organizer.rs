//! Organizers of private events (ADR 0061): the creator, and the people an organizer granted
//! it to. The seed says whether the event is private and whether organizers alone decide, so
//! these rules need no lookups beyond the cited grant.

use hdi::prelude::*;
use sp_event::Audience;

use crate::{creator, invalid, EntryTypes};

/// A co-organizer. The creator grants with `grant: None`; another organizer cites their own
/// `Organizer` record.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Organizer {
    pub agent: AgentPubKey,
    pub grant: Option<ActionHash>,
}

/// Why someone is an organizer; coordinators cite `grant()` in what they write.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum OrganizerRole {
    Creator,
    Granted(ActionHash),
}

impl OrganizerRole {
    #[must_use]
    pub fn grant(self) -> Option<ActionHash> {
        match self {
            Self::Creator => None,
            Self::Granted(hash) => Some(hash),
        }
    }
}

/// The audience this clone's seed names.
pub fn audience() -> ExternResult<Audience> {
    Ok(Audience::of_seed(&dna_info()?.modifiers.network_seed))
}

/// Whether organizers alone decide here (an organizer-controlled event).
pub fn organized() -> ExternResult<bool> {
    Ok(audience()?.organized())
}

pub fn validate(
    o: &Organizer,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    if !audience()?.is_private() {
        return invalid("only private events have co-organizers");
    }
    if creator()?.as_ref() == Some(&o.agent) {
        return invalid("the creator is already an organizer");
    }
    if is_organizer(action.author(), o.grant.as_ref())? {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid("only organizers can add organizers")
    }
}

/// `agent` is the creator, or `grant` is a valid `Organizer` record naming them.
pub fn is_organizer(agent: &AgentPubKey, grant: Option<&ActionHash>) -> ExternResult<bool> {
    if creator()?.as_ref() == Some(agent) {
        return Ok(true);
    }
    let Some(grant) = grant else {
        return Ok(false);
    };
    let record = must_get_valid_record(grant.clone())?;
    Ok(matches!(
        crate::typed_entry(&record)?,
        Some(EntryTypes::Organizer(o)) if &o.agent == agent
    ))
}

/// In an organizer-controlled event, `agent` must be an organizer to do `what`.
pub fn require_in_organized(
    agent: &AgentPubKey,
    grant: Option<&ActionHash>,
    what: &str,
) -> ExternResult<Option<ValidateCallbackResult>> {
    if organized()? && !is_organizer(agent, grant)? {
        return Ok(Some(ValidateCallbackResult::Invalid(format!(
            "only organizers can {what} in this event"
        ))));
    }
    Ok(None)
}
