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
    if let Some(reason) = voucher_not_full(&v.voucher_standing, action)? {
        return invalid(&reason);
    }
    let earlier = earlier_vouch_times(action)?;
    match check_vouch_cap(&earlier, action.timestamp().as_micros()) {
        Ok(()) => Ok(ValidateCallbackResult::Valid),
        Err(e) => invalid(&e.to_string()),
    }
}

/// `None` when the proof shows the author was Full before this action.
fn voucher_not_full(
    proof: &StandingProof,
    action: &TypedAction<CreateData>,
) -> ExternResult<Option<String>> {
    match proof {
        StandingProof::Founder if is_founder(action.author())? => Ok(None),
        StandingProof::Founder => Ok(Some("voucher is not a founder".into())),
        StandingProof::Grant(hash) => {
            let record = must_get_valid_record(hash.clone())?;
            let is_grant = matches!(typed_entry(&record)?, Some(EntryTypes::StandingGrant(_)));
            if !is_grant || record.action().author() != action.author() {
                return Ok(Some(
                    "voucher's standing proof is not their own grant".into(),
                ));
            }
            if record.action().timestamp() >= action.timestamp() {
                return Ok(Some("voucher's grant is newer than the vouch".into()));
            }
            Ok(None)
        }
    }
}

/// Times of the author's vouches in the 30 days before this action, from their own chain.
fn earlier_vouch_times(action: &TypedAction<CreateData>) -> ExternResult<Vec<i64>> {
    let since = Timestamp::from_micros(action.timestamp().as_micros() - VOUCH_CAP_WINDOW_US);
    let filter = ChainFilter::until_timestamp(crate::prev_action(action)?, since);
    let vouch_type = EntryType::App(UnitEntryTypes::Vouch.try_into()?);
    let activity = must_get_agent_activity(action.author().clone(), filter)?;
    Ok(activity
        .iter()
        .map(|a| a.action.hashed.content.clone())
        .filter(|a| a.entry_type() == Some(&vouch_type) && matches!(a.data, ActionData::Create(_)))
        .map(|a| a.timestamp().as_micros())
        .collect())
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
