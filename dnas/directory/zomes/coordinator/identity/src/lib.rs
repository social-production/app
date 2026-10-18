//! Identity API (directory DNA). Used by `ui/src/lib/services/people.ts` and the scenario
//! tests. Reads are best-effort views; integrity validation is what enforces the rules.

mod devices;
mod profile;
mod records;
mod standing;

use hdk::prelude::*;
use identity_integrity::{founder_agents, DirectoryProperties, Founder};

pub use profile::ProfileInput;
pub use standing::StandingView;

#[hdk_extern]
pub fn my_agent(_: ()) -> ExternResult<AgentPubKey> {
    Ok(agent_info()?.agent_initial_pubkey)
}

#[hdk_extern]
pub fn founders(_: ()) -> ExternResult<Vec<Founder>> {
    Ok(DirectoryProperties::try_from_dna_properties()?.founders)
}

#[hdk_extern]
pub fn set_profile(input: ProfileInput) -> ExternResult<ActionHash> {
    profile::set(input)
}

#[hdk_extern]
pub fn get_profile(agent: AgentPubKey) -> ExternResult<Option<identity_integrity::Profile>> {
    profile::get(&agent)
}

#[hdk_extern]
pub fn list_people(_: ()) -> ExternResult<Vec<AgentPubKey>> {
    profile::list_people()
}

#[hdk_extern]
pub fn get_standing(agent: AgentPubKey) -> ExternResult<StandingView> {
    standing::view(&agent, &founder_agents()?)
}

#[hdk_extern]
pub fn vouch(vouchee: AgentPubKey) -> ExternResult<ActionHash> {
    standing::vouch(vouchee)
}

#[hdk_extern]
pub fn withdraw_vouch(vouch: ActionHash) -> ExternResult<ActionHash> {
    delete_entry(vouch)
}

#[hdk_extern]
pub fn claim_full_standing(_: ()) -> ExternResult<ActionHash> {
    standing::claim()
}

#[hdk_extern]
pub fn link_device(link: identity_integrity::DeviceLink) -> ExternResult<ActionHash> {
    devices::link(link)
}

#[hdk_extern]
pub fn get_devices(agent: AgentPubKey) -> ExternResult<Vec<AgentPubKey>> {
    devices::confirmed(&agent)
}

/// Write a vouch exactly as given, skipping the friendly checks in `vouch`. The rules are
/// enforced by integrity validation, not here; tests use this to prove forged vouches fail.
#[hdk_extern]
pub fn submit_vouch(v: identity_integrity::Vouch) -> ExternResult<ActionHash> {
    create_entry(&identity_integrity::EntryTypes::Vouch(v))
}

/// Write a standing grant with the given evidence (see `submit_vouch`).
#[hdk_extern]
pub fn submit_grant(evidence: identity_integrity::Evidence) -> ExternResult<ActionHash> {
    create_entry(&identity_integrity::EntryTypes::StandingGrant(
        identity_integrity::StandingGrant { evidence },
    ))
}

pub(crate) fn guest_error(message: impl Into<String>) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(message.into()))
}
