//! M1 identity rules end to end (docs/GOVERNANCE.md#standing).

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetAgents, SweetConductorBatch};
use identity_integrity::{DeviceLink, Profile};
use serde::{Deserialize, Serialize};
use sp_governance::standing::StandingLevel;

mod common;

#[derive(Serialize, Deserialize, Debug)]
struct ProfileInput {
    name: String,
    bio: String,
    home_area: String,
    avatar: Option<identity_integrity::Avatar>,
}

#[derive(Deserialize, Debug)]
struct StandingView {
    level: StandingLevel,
    founder: bool,
    grant: Option<ActionHash>,
    vouchers: Vec<AgentPubKey>,
}

fn profile(name: &str) -> ProfileInput {
    ProfileInput {
        name: name.into(),
        bio: String::new(),
        home_area: "Brisbane".into(),
        avatar: None,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn two_founder_vouches_grant_full_standing() {
    let mut conductors = SweetConductorBatch::standard(3).await;
    let cells = common::directory_network(&mut conductors, 2).await;
    let (ann, bob, eve) = (
        cells[0].zome("identity"),
        cells[1].zome("identity"),
        cells[2].zome("identity"),
    );
    let eve_key = cells[2].agent_pubkey().clone();

    let founder: StandingView = conductors[0]
        .call(&ann, "get_standing", cells[0].agent_pubkey().clone())
        .await;
    assert!(founder.founder && founder.level == StandingLevel::Full);

    let _: ActionHash = conductors[0].call(&ann, "vouch", eve_key.clone()).await;
    await_consistency(&cells).await.expect("consistency");
    let early: Result<ActionHash, _> = conductors[2]
        .call_fallible(&eve, "claim_full_standing", ())
        .await;
    assert!(early.is_err(), "one vouch is not enough");

    let _: ActionHash = conductors[1].call(&bob, "vouch", eve_key.clone()).await;
    await_consistency(&cells).await.expect("consistency");
    let _: ActionHash = conductors[2].call(&eve, "claim_full_standing", ()).await;
    await_consistency(&cells).await.expect("consistency");

    let seen: StandingView = conductors[0].call(&ann, "get_standing", eve_key).await;
    assert_eq!(seen.level, StandingLevel::Full);
    assert!(seen.grant.is_some() && !seen.founder);
    assert_eq!(seen.vouchers.len(), 2);

    let stranger = SweetAgents::one(conductors[2].keystore()).await;
    let vouched: Result<ActionHash, _> = conductors[2].call_fallible(&eve, "vouch", stranger).await;
    assert!(vouched.is_ok(), "a member with a valid grant can vouch");
    common::shut_down(conductors).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn validation_rejects_forged_standing() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::directory_network(&mut conductors, 1).await;
    let (ann, mal) = (cells[0].zome("identity"), cells[1].zome("identity"));
    let mal_key = cells[1].agent_pubkey().clone();
    let stranger = SweetAgents::one(conductors[1].keystore()).await;

    let fake_founder = identity_integrity::Vouch {
        vouchee: stranger,
        voucher_standing: identity_integrity::StandingProof::Founder,
    };
    let r: Result<ActionHash, _> = conductors[1]
        .call_fallible(&mal, "submit_vouch", fake_founder)
        .await;
    assert!(r.is_err(), "non-founders cannot claim founder standing");

    let vouch: ActionHash = conductors[0].call(&ann, "vouch", mal_key).await;
    await_consistency(&cells).await.expect("consistency");
    let doubled = identity_integrity::Evidence::Vouches(vec![vouch.clone(), vouch]);
    let r: Result<ActionHash, _> = conductors[1]
        .call_fallible(&mal, "submit_grant", doubled)
        .await;
    assert!(r.is_err(), "one voucher cited twice is not two vouchers");

    let steady = identity_integrity::Evidence::SteadyActivity;
    let r: Result<ActionHash, _> = conductors[1]
        .call_fallible(&mal, "submit_grant", steady)
        .await;
    assert!(r.is_err(), "a day-old chain is not steady activity");
    common::shut_down(conductors).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn vouch_cap_profiles_and_device_pairing() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::directory_network(&mut conductors, 1).await;
    let (ann, bea) = (cells[0].zome("identity"), cells[1].zome("identity"));
    let (ann_key, bea_key) = (
        cells[0].agent_pubkey().clone(),
        cells[1].agent_pubkey().clone(),
    );

    for _ in 0..5 {
        let someone = SweetAgents::one(conductors[0].keystore()).await;
        let _: ActionHash = conductors[0].call(&ann, "vouch", someone).await;
    }
    let sixth = SweetAgents::one(conductors[0].keystore()).await;
    let r: Result<ActionHash, _> = conductors[0].call_fallible(&ann, "vouch", sixth).await;
    assert!(r.is_err(), "five vouches per 30 days");

    let r: Result<ActionHash, _> = conductors[1]
        .call_fallible(&bea, "set_profile", profile("  "))
        .await;
    assert!(r.is_err(), "blank names are rejected");
    let r: Result<ActionHash, _> = conductors[1]
        .call_fallible(&bea, "set_profile", profile("Bea Smith"))
        .await;
    assert!(r.is_err(), "usernames have no spaces (web's handle rules)");
    let _: ActionHash = conductors[1]
        .call(&bea, "set_profile", profile("Bea"))
        .await;
    let _: ActionHash = conductors[1]
        .call(&bea, "set_profile", profile("bea_smith"))
        .await;
    await_consistency(&cells).await.expect("consistency");
    let seen: Option<Profile> = conductors[0]
        .call(&ann, "get_profile", bea_key.clone())
        .await;
    assert_eq!(
        seen.map(|p| p.name).as_deref(),
        Some("bea_smith"),
        "latest update wins"
    );

    let link = DeviceLink {
        primary: ann_key.clone(),
        device: bea_key.clone(),
    };
    let _: ActionHash = conductors[0].call(&ann, "link_device", link.clone()).await;
    await_consistency(&cells).await.expect("consistency");
    let half: Vec<AgentPubKey> = conductors[1]
        .call(&bea, "get_devices", ann_key.clone())
        .await;
    assert!(half.is_empty(), "a pairing needs both halves");
    let _: ActionHash = conductors[1].call(&bea, "link_device", link).await;
    await_consistency(&cells).await.expect("consistency");
    let paired: Vec<AgentPubKey> = conductors[0]
        .call(&ann, "get_devices", ann_key.clone())
        .await;
    assert_eq!(paired, vec![bea_key.clone()]);
    let primary: Pairing = conductors[0]
        .call(&ann, "get_pairing", ann_key.clone())
        .await;
    assert_eq!(primary.devices, vec![bea_key.clone()]);
    let device: Pairing = conductors[0].call(&ann, "get_pairing", bea_key).await;
    assert_eq!(device.primary, Some(ann_key), "the device knows its person");
    common::shut_down(conductors).await;
}

#[derive(Deserialize, Debug)]
struct Pairing {
    primary: Option<AgentPubKey>,
    devices: Vec<AgentPubKey>,
}

/// The founder keys written in dnas/directory/dna.yaml parse and count as Full. A malformed
/// key there would make every validation that reads the founders fail.
#[tokio::test(flavor = "multi_thread")]
async fn manifest_founders_are_valid_and_full() {
    let dna = common::load_dna("directory").await;
    let mut conductors = SweetConductorBatch::standard(1).await;
    let apps = conductors
        .setup_app("sp", &[dna])
        .await
        .expect("install app");
    let zome = apps.cells_flattened()[0].zome("identity");

    let founders: Vec<identity_integrity::Founder> =
        conductors[0].call(&zome, "founders", ()).await;
    assert_eq!(founders.len(), 2);
    for founder in founders {
        let key: AgentPubKey = holo_hash::AgentPubKeyB64::from_b64_str(&founder.agent)
            .expect("valid founder key")
            .into();
        let standing: StandingView = conductors[0].call(&zome, "get_standing", key).await;
        assert!(
            standing.founder && standing.level == StandingLevel::Full,
            "{}",
            founder.name
        );
    }
    common::shut_down(conductors).await;
}
