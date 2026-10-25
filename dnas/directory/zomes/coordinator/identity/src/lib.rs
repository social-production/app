//! Identity API (directory DNA). Used by `ui/src/lib/services/people.ts` and the scenario
//! tests. Reads are best-effort views; integrity validation is what enforces the rules.

mod devices;
mod feedback;
mod follow_links;
mod follows;
mod listings;
mod profile;
mod records;
mod standing;

use hdk::prelude::*;
use identity_integrity::{founder_agents, DirectoryProperties, Founder};

pub use feedback::{FeedbackView, FeedbackVote, VoteDirection};
pub use follows::{FollowRow, FollowStatus, FollowView};
pub use listings::{Allowance, ListingInput, ListingView};
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

#[derive(Serialize, Deserialize, Debug)]
pub struct StandingAt {
    pub agent: AgentPubKey,
    pub at: Timestamp,
}

#[hdk_extern]
pub fn full_standing_before(query: StandingAt) -> ExternResult<bool> {
    standing::full_before(&query.agent, query.at)
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
pub fn create_listing(input: ListingInput) -> ExternResult<ActionHash> {
    listings::create(input)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListingUpdate {
    pub id: ActionHash,
    pub listing: ListingInput,
}

#[hdk_extern]
pub fn update_listing(input: ListingUpdate) -> ExternResult<ActionHash> {
    listings::update(input.id, input.listing)
}

#[hdk_extern]
pub fn list_projects(_: ()) -> ExternResult<Vec<ListingView>> {
    listings::list()
}

#[hdk_extern]
pub fn get_listing(id: ActionHash) -> ExternResult<Option<ListingView>> {
    listings::get(id)
}

#[hdk_extern]
pub fn find_listing(network_seed: String) -> ExternResult<Option<ListingView>> {
    listings::by_seed(network_seed)
}

#[hdk_extern]
pub fn project_allowance(_: ()) -> ExternResult<Allowance> {
    listings::allowance()
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

/// Whose device `agent` is, and which devices it has (for showing one person per row).
#[hdk_extern]
pub fn get_pairing(agent: AgentPubKey) -> ExternResult<devices::Pairing> {
    devices::pairing(&agent)
}

/// Pairings other devices started with me, waiting for my confirmation.
#[hdk_extern]
pub fn device_requests(_: ()) -> ExternResult<Vec<identity_integrity::DeviceLink>> {
    devices::requests()
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

/// Follow someone; returns the follow's status (pending while they require approval).
#[hdk_extern]
pub fn follow(target: AgentPubKey) -> ExternResult<Option<FollowStatus>> {
    follows::follow(target)
}

#[hdk_extern]
pub fn unfollow(target: AgentPubKey) -> ExternResult<()> {
    follows::unfollow(target)
}

#[hdk_extern]
pub fn accept_follow(follower: AgentPubKey) -> ExternResult<ActionHash> {
    follows::decide(follower, true)
}

#[hdk_extern]
pub fn reject_follow(follower: AgentPubKey) -> ExternResult<ActionHash> {
    follows::decide(follower, false)
}

#[hdk_extern]
pub fn set_follow_approval(on: bool) -> ExternResult<bool> {
    follows::set_approval(on)
}

#[hdk_extern]
pub fn follow_view(agent: AgentPubKey) -> ExternResult<FollowView> {
    follows::view(agent)
}

#[hdk_extern]
pub fn follow_requests(_: ()) -> ExternResult<Vec<FollowRow>> {
    follows::requests()
}

#[hdk_extern]
pub fn create_feedback(input: identity_integrity::Feedback) -> ExternResult<FeedbackView> {
    feedback::create(input)
}

/// Every feedback item, newest first.
#[hdk_extern]
pub fn list_feedback(_: ()) -> ExternResult<Vec<FeedbackView>> {
    feedback::list()
}

#[hdk_extern]
pub fn get_feedback(id: ActionHash) -> ExternResult<Option<FeedbackView>> {
    feedback::get(id)
}

#[hdk_extern]
pub fn vote_feedback(input: FeedbackVote) -> ExternResult<Option<FeedbackView>> {
    feedback::vote(input)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawFollowLink {
    pub base: AgentPubKey,
    pub target: AgentPubKey,
}

/// Write a `Follows` link exactly as given (see `submit_vouch`): tests prove forged ones fail.
#[hdk_extern]
pub fn submit_follow_link(link: RawFollowLink) -> ExternResult<ActionHash> {
    create_link(
        link.base,
        link.target,
        identity_integrity::LinkTypes::Follows,
        (),
    )
}

pub(crate) fn guest_error(message: impl Into<String>) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(message.into()))
}
