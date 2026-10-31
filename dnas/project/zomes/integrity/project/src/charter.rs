//! The charter founds a space (ADR 0065): anyone may found one in the commons; in a private
//! clone only the creator its seed names may. From M3 it changes only through passed `Edit`
//! proposals, applied at read time (ADR 0033).

use hdi::prelude::*;
use sp_core::project::{check_fields, CharterFields, DESCRIPTION_MAX, MODES, SUBTYPES};

use crate::{creator, invalid, Charter};

pub fn validate_create(c: &Charter, author: &AgentPubKey) -> ExternResult<ValidateCallbackResult> {
    if creator()?.is_some_and(|creator| &creator != author) {
        return invalid("only the creator its seed names can found this private space");
    }
    match check_charter(c) {
        Ok(()) => Ok(ValidateCallbackResult::Valid),
        Err(reason) => invalid(&reason),
    }
}

/// The "charter" anchor link (how a private clone finds its one space) points at a charter
/// its author wrote.
pub fn validate_anchor(
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    let founder = match action.target_address.clone().into_action_hash() {
        Some(hash) => crate::space_creator(&hash)?,
        None => None,
    };
    if founder.as_ref() == Some(action.author()) {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid("only a space's creator links its charter")
    }
}

fn check_charter(c: &Charter) -> Result<(), String> {
    if c.description.trim().is_empty() || c.description.chars().count() > DESCRIPTION_MAX {
        return Err(format!(
            "description must be 1–{DESCRIPTION_MAX} characters"
        ));
    }
    // An event (ADR 0057), help request (ADR 0067), thread or post (ADR 0070) checks its kind here and the shared
    // field limits below.
    let (mode, subtype) = if c.mode == sp_event::MODE {
        sp_event::check_kind(&c.mode, &c.subtype)?;
        (MODES[0], SUBTYPES[0])
    } else if c.mode == sp_help::MODE {
        sp_help::check_kind(&c.mode, &c.subtype)?;
        (MODES[0], SUBTYPES[0])
    } else if sp_discussion::is_discussion(&c.mode) {
        // A thread or post (ADR 0070).
        sp_discussion::check_kind(&c.mode, &c.subtype)?;
        (MODES[0], SUBTYPES[0])
    } else if sp_scope::is_scope(&c.mode) {
        check_scope_charter(c)?;
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

/// A channel or community (ADR 0068): the title is its handle, the subtype its join policy.
/// A closed community lives in its own clone (a private seed); public scopes in the commons.
fn check_scope_charter(c: &Charter) -> Result<(), String> {
    sp_scope::check_scope(&c.mode, &c.title, &c.subtype)?;
    let in_clone = creator().map_err(|e| e.to_string())?.is_some();
    match (c.subtype == "closed", in_clone) {
        (true, true) | (false, false) => Ok(()),
        (true, false) => Err("a closed community has its own private network".into()),
        (false, true) => Err("only a closed community lives in a private network".into()),
    }
}
