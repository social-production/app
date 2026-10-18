use hdi::prelude::*;
use sp_governance::standing::{check_steady_activity, check_vouch_evidence, VouchFact};

use crate::{invalid, vouch, Evidence, StandingGrant};

pub fn validate_create(
    g: &StandingGrant,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    let at = action.timestamp().as_micros();
    let result = match &g.evidence {
        Evidence::Vouches(hashes) => {
            let Some(facts) = load_vouches(hashes)? else {
                return invalid("a cited record is not a vouch");
            };
            check_vouch_evidence(action.author(), &facts, at)
        }
        Evidence::SteadyActivity => {
            let (start, times) = own_chain_times(action)?;
            check_steady_activity(start, &times, at)
        }
    };
    match result {
        Ok(()) => Ok(ValidateCallbackResult::Valid),
        Err(e) => invalid(&e.to_string()),
    }
}

/// Each cited vouch must itself be valid, which already proved its voucher was Full.
fn load_vouches(hashes: &[ActionHash]) -> ExternResult<Option<Vec<VouchFact<AgentPubKey>>>> {
    let mut facts = Vec::with_capacity(hashes.len());
    for hash in hashes {
        let Some((voucher, v, at)) = vouch::load(hash)? else {
            return Ok(None);
        };
        facts.push(VouchFact {
            voucher,
            vouchee: v.vouchee,
            at,
        });
    }
    Ok(Some(facts))
}

/// First action time and every action time on the author's chain before this grant.
fn own_chain_times(action: &TypedAction<CreateData>) -> ExternResult<(i64, Vec<i64>)> {
    let filter = ChainFilter::new(crate::prev_action(action)?);
    let activity = must_get_agent_activity(action.author().clone(), filter)?;
    let times: Vec<i64> = activity
        .iter()
        .map(|a| a.action.hashed.content.timestamp().as_micros())
        .collect();
    let start = times
        .iter()
        .copied()
        .min()
        .unwrap_or(action.timestamp().as_micros());
    Ok((start, times))
}
