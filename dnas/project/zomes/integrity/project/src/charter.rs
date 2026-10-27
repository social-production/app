//! The charter: written once by the creator; from M3 it changes only through passed `Edit`
//! proposals, applied at read time (ADR 0033).

use hdi::prelude::*;
use sp_core::project::{check_fields, CharterFields, DESCRIPTION_MAX, MODES, SUBTYPES};

use crate::{creator, invalid, Charter};

pub fn validate_create(c: &Charter, author: &AgentPubKey) -> ExternResult<ValidateCallbackResult> {
    by_creator(author, || check_charter(c))
}

/// Only the creator writes the charter (and links it from the "charter" anchor).
pub fn by_creator(
    author: &AgentPubKey,
    then: impl FnOnce() -> Result<(), String>,
) -> ExternResult<ValidateCallbackResult> {
    if creator()?.as_ref() != Some(author) {
        return invalid("only the project's creator can write its charter");
    }
    match then() {
        Ok(()) => Ok(ValidateCallbackResult::Valid),
        Err(reason) => invalid(&reason),
    }
}

fn check_charter(c: &Charter) -> Result<(), String> {
    if c.description.trim().is_empty() || c.description.chars().count() > DESCRIPTION_MAX {
        return Err(format!(
            "description must be 1–{DESCRIPTION_MAX} characters"
        ));
    }
    // An event (ADR 0057) checks its kind here and the shared field limits below.
    let (mode, subtype) = if c.mode == sp_event::MODE {
        sp_event::check_kind(&c.mode, &c.subtype)?;
        (MODES[0], SUBTYPES[0])
    } else {
        (c.mode.as_str(), c.subtype.as_str())
    };
    check_fields(&CharterFields {
        title: &c.title,
        mode,
        subtype,
        location_label: &c.location_label,
        phase: &c.phase,
        tags: &c.tags,
    })
}
