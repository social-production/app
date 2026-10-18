use hdk::prelude::*;
use identity_integrity::{
    founder_agents, EntryTypes, Evidence, LinkTypes, StandingGrant, StandingProof, UnitEntryTypes,
    Vouch,
};
use sp_governance::standing::{
    check_steady_activity, check_vouch_cap, StandingLevel, VOUCHES_FOR_FULL,
};

use crate::guest_error;
use crate::records::{action_targets, decode, links, live};

#[derive(Serialize, Deserialize, Debug)]
pub struct StandingView {
    pub level: StandingLevel,
    pub founder: bool,
    pub grant: Option<ActionHash>,
    /// Distinct agents with a live vouch for this agent.
    pub vouchers: Vec<AgentPubKey>,
}

pub fn view(agent: &AgentPubKey, founders: &[AgentPubKey]) -> ExternResult<StandingView> {
    let founder = founders.contains(agent);
    let grant = grant_of(agent)?;
    let level = if founder || grant.is_some() {
        StandingLevel::Full
    } else {
        StandingLevel::New
    };
    let vouchers = vouches_for(agent)?
        .into_iter()
        .map(|(_, voucher)| voucher)
        .collect();
    Ok(StandingView {
        level,
        founder,
        grant,
        vouchers,
    })
}

/// Vouch for `vouchee`. Fails early with a readable reason when validation would reject it.
pub fn vouch(vouchee: AgentPubKey) -> ExternResult<ActionHash> {
    let me = agent_info()?.agent_initial_pubkey;
    if vouchee == me {
        return Err(guest_error("You can't vouch for yourself."));
    }
    let voucher_standing = if founder_agents()?.contains(&me) {
        StandingProof::Founder
    } else {
        let grant = grant_of(&me)?.ok_or_else(|| guest_error("Only full members can vouch."))?;
        StandingProof::Grant(grant)
    };
    let earlier = own_entry_times(UnitEntryTypes::Vouch)?;
    check_vouch_cap(&earlier, sys_time()?.as_micros()).map_err(|e| guest_error(e.to_string()))?;

    let hash = create_entry(&EntryTypes::Vouch(Vouch {
        vouchee: vouchee.clone(),
        voucher_standing,
    }))?;
    create_link(vouchee, hash.clone(), LinkTypes::VoucheeToVouches, ())?;
    Ok(hash)
}

/// Claim Full standing from vouches received, else from steady activity.
pub fn claim() -> ExternResult<ActionHash> {
    let me = agent_info()?.agent_initial_pubkey;
    if grant_of(&me)?.is_some() {
        return Err(guest_error("You already have full standing."));
    }
    let vouches = vouches_for(&me)?;
    let evidence = if vouches.len() >= VOUCHES_FOR_FULL {
        Evidence::Vouches(
            vouches
                .into_iter()
                .take(VOUCHES_FOR_FULL)
                .map(|(h, _)| h)
                .collect(),
        )
    } else {
        let times = own_action_times()?;
        let start = times.iter().copied().min().unwrap_or_default();
        check_steady_activity(start, &times, sys_time()?.as_micros()).map_err(|e| {
            guest_error(format!(
                "Not yet: {e}. Or ask {VOUCHES_FOR_FULL} full members to vouch."
            ))
        })?;
        Evidence::SteadyActivity
    };
    let hash = create_entry(&EntryTypes::StandingGrant(StandingGrant { evidence }))?;
    create_link(me, hash.clone(), LinkTypes::AgentToGrants, ())?;
    Ok(hash)
}

fn grant_of(agent: &AgentPubKey) -> ExternResult<Option<ActionHash>> {
    for hash in action_targets(links(agent.clone(), LinkTypes::AgentToGrants)?) {
        if let Some(record) = live(hash.clone())? {
            if decode::<StandingGrant>(&record)?.is_some() {
                return Ok(Some(hash));
            }
        }
    }
    Ok(None)
}

/// (vouch action, voucher) for each live vouch, one per distinct voucher, oldest first.
fn vouches_for(agent: &AgentPubKey) -> ExternResult<Vec<(ActionHash, AgentPubKey)>> {
    let mut found = links(agent.clone(), LinkTypes::VoucheeToVouches)?;
    found.sort_by_key(|l| l.timestamp);
    let mut out: Vec<(ActionHash, AgentPubKey)> = Vec::new();
    for hash in action_targets(found) {
        let Some(record) = live(hash.clone())? else {
            continue;
        };
        let Some(v) = decode::<Vouch>(&record)? else {
            continue;
        };
        let voucher = record.action().author().clone();
        if &v.vouchee == agent && !out.iter().any(|(_, a)| a == &voucher) {
            out.push((hash, voucher));
        }
    }
    Ok(out)
}

fn own_entry_times(unit: UnitEntryTypes) -> ExternResult<Vec<i64>> {
    let entry_type = EntryType::App(unit.try_into()?);
    let filter = ChainQueryFilter::new()
        .entry_type(entry_type)
        .action_type(ActionType::Create);
    Ok(query(filter)?
        .iter()
        .map(|r| r.action().timestamp().as_micros())
        .collect())
}

fn own_action_times() -> ExternResult<Vec<i64>> {
    Ok(query(ChainQueryFilter::new())?
        .iter()
        .map(|r| r.action().timestamp().as_micros())
        .collect())
}
