use hdi::prelude::*;
use sp_governance::standing::{check_vouch_cap, VOUCH_CAP_WINDOW_US};

use crate::founders::is_founder;
use crate::{invalid, typed_entry, EntryTypes, StandingProof, UnitEntryTypes, Vouch};

/// A vouch is valid when the voucher is Full at the time and under the 30-day cap.
pub fn validate_create(
    v: &Vouch,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    if &v.vouchee == action.author() {
        return invalid("agents cannot vouch for themselves");
    }
    if let Some(reason) = not_full(&v.voucher_standing, action)? {
        return invalid(&format!("voucher: {reason}"));
    }
    let earlier = crate::earlier_creates(action, UnitEntryTypes::Vouch, VOUCH_CAP_WINDOW_US)?;
    match check_vouch_cap(&earlier, action.timestamp().as_micros()) {
        Ok(()) => Ok(ValidateCallbackResult::Valid),
        Err(e) => invalid(&e.to_string()),
    }
}

/// `None` when the proof shows the author was Full before this action.
pub(crate) fn not_full(
    proof: &StandingProof,
    action: &TypedAction<CreateData>,
) -> ExternResult<Option<String>> {
    match proof {
        StandingProof::Founder if is_founder(action.author())? => Ok(None),
        StandingProof::Founder => Ok(Some("not a founder".into())),
        StandingProof::Grant(hash) => {
            let record = must_get_valid_record(hash.clone())?;
            let is_grant = matches!(typed_entry(&record)?, Some(EntryTypes::StandingGrant(_)));
            if !is_grant || record.action().author() != action.author() {
                return Ok(Some("standing proof is not their own grant".into()));
            }
            if record.action().timestamp() >= action.timestamp() {
                return Ok(Some("standing grant is newer than this action".into()));
            }
            Ok(None)
        }
    }
}

/// Read a cited vouch record (used by standing grants).
pub fn load(hash: &ActionHash) -> ExternResult<Option<(AgentPubKey, Vouch, i64)>> {
    let record = must_get_valid_record(hash.clone())?;
    let Some(EntryTypes::Vouch(vouch)) = typed_entry(&record)? else {
        return Ok(None);
    };
    let author = record.action().author().clone();
    Ok(Some((
        author,
        vouch,
        record.action().timestamp().as_micros(),
    )))
}
