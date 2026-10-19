use hdi::prelude::*;
use holo_hash::AgentPubKeyB64;
use sp_core::project::{check_fields, CharterFields, SUMMARY_MAX};
use sp_governance::projects::{check_new_member_project_cap, NEW_PROJECT_WINDOW_US};

use crate::vouch::not_full;
use crate::{invalid, typed_entry, EntryTypes, ProjectListing, UnitEntryTypes};

/// `<creator key>.<nonce>` (see `project_integrity::creator_in_seed`).
const NETWORK_SEED_MAX: usize = 128;

/// Valid fields, and either a Full-standing proof or room under the New-member limit.
pub fn validate_create(
    l: &ProjectListing,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    if let Err(reason) = check(l) {
        return invalid(&reason);
    }
    let own_seed = format!("{}.", AgentPubKeyB64::from(action.author().clone()));
    if !l.network_seed.starts_with(&own_seed) {
        return invalid("a listing's project must name its author as creator");
    }
    if let Some(proof) = &l.creator_standing {
        return match not_full(proof, action)? {
            Some(reason) => invalid(&format!("creator: {reason}")),
            None => Ok(ValidateCallbackResult::Valid),
        };
    }
    let window = NEW_PROJECT_WINDOW_US;
    let earlier = crate::earlier_creates(action, UnitEntryTypes::ProjectListing, window)?;
    match check_new_member_project_cap(&earlier, action.timestamp().as_micros()) {
        Ok(()) => Ok(ValidateCallbackResult::Valid),
        Err(reason) => invalid(&reason),
    }
}

/// The creator keeps the listing in step with the charter; the project it points to never changes.
pub fn validate_update(
    l: &ProjectListing,
    action: &TypedAction<UpdateData>,
) -> ExternResult<ValidateCallbackResult> {
    let original = must_get_valid_record(action.original_action_address.clone())?;
    if original.action().author() != action.author() {
        return invalid("only the project's creator can update its listing");
    }
    let Some(EntryTypes::ProjectListing(before)) = typed_entry(&original)? else {
        return invalid("a listing can only update a listing");
    };
    if before.network_seed != l.network_seed {
        return invalid("a listing cannot move to another project");
    }
    match check(l) {
        Ok(()) => Ok(ValidateCallbackResult::Valid),
        Err(reason) => invalid(&reason),
    }
}

fn check(l: &ProjectListing) -> Result<(), String> {
    if l.network_seed.len() > NETWORK_SEED_MAX || l.network_seed.split_once('.').is_none() {
        return Err("a listing needs its project's network seed".into());
    }
    if l.summary.chars().count() > SUMMARY_MAX {
        return Err(format!("summary must be at most {SUMMARY_MAX} characters"));
    }
    check_fields(&CharterFields {
        title: &l.title,
        mode: &l.mode,
        subtype: &l.subtype,
        location_label: &l.location_label,
        phase: &l.phase,
        tags: &l.tags,
    })
}
