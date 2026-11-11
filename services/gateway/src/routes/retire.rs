//! Moving a hosted account to the desktop app (ADR 0092). Once the desktop has the person's
//! chain, it signs a short statement with the phrase's key and sends it here; the gateway
//! removes its hosted copy (one agent on two nodes can fork its chain) and refuses to host
//! it again until the person asks for it back with their words (`reclaim` in `account::open`).
//! No secret leaves the desktop: the signature proves the request comes from the phrase.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use holo_hash::AgentPubKeyB64;
use holochain::prelude::AgentPubKey;
use serde::{Deserialize, Serialize};

use super::{client_ip, ApiError, ApiResult, AppState};

/// How far a statement's time may be from ours (clock drift and a slow network).
const WINDOW_SECS: i64 = 10 * 60;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetireRequest {
    /// The person's agent key (base64, `uhCAk…`).
    agent: String,
    /// Unix seconds when the desktop signed.
    at: i64,
    /// ed25519 signature of [`statement`] by the agent key.
    signature: Vec<u8>,
}

#[derive(Serialize)]
pub struct Retired {
    /// Whether this gateway hosted the account (and has now stopped).
    retired: bool,
}

/// The bytes the desktop signs (`native/src/retire.rs` writes the same).
pub fn statement(agent: &str, at: i64) -> Vec<u8> {
    format!("social-production: retire the hosted copy of {agent} at {at}").into_bytes()
}

/// Whether `signature` is the agent's own signature of the statement for `at`.
pub fn verify(agent: &AgentPubKey, at: i64, signature: &[u8]) -> bool {
    let Ok(signature) = <&[u8; sodoken::sign::SIGNATUREBYTES]>::try_from(signature) else {
        return false;
    };
    let Ok(public) = <&[u8; sodoken::sign::PUBLICKEYBYTES]>::try_from(agent.get_raw_32()) else {
        return false;
    };
    let b64 = AgentPubKeyB64::from(agent.clone()).to_string();
    sodoken::sign::verify_detached(signature, &statement(&b64, at), public)
}

pub async fn retire(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RetireRequest>,
) -> ApiResult<Retired> {
    if !state.signin_limit.allow(&client_ip(&headers)) {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests from this network today.".into(),
        ));
    }
    let agent = AgentPubKeyB64::from_b64_str(&req.agent)
        .map(AgentPubKey::from)
        .map_err(|_| ApiError::bad_request("agent is not an agent key"))?;
    if (now() - req.at).abs() > WINDOW_SECS {
        return Err(ApiError::bad_request(
            "This computer's clock is too far off. Check the date and time.",
        ));
    }
    if !verify(&agent, req.at, &req.signature) {
        return Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "The signature doesn't match this account.".into(),
        ));
    }
    if !state.store.person_exists(&req.agent).await? {
        return Ok(Json(Retired { retired: false }));
    }
    state.store.set_retired(&req.agent, true).await?;
    let removed = Box::pin(state.node.retire_hosted(&agent)).await?;
    tracing::info!(agent = %req.agent, removed, "hosted account moved to a desktop");
    Ok(Json(Retired { retired: true }))
}

/// The refusal for an account that moved (HTTP 410, which the UI recognises).
pub fn moved(with_phrase: bool) -> ApiError {
    let how = if with_phrase {
        "Use it there, or choose “Use it here instead” if you no longer use it on that computer."
    } else {
        "Use it there, or log in with your recovery phrase to use it here instead."
    };
    ApiError(
        StatusCode::GONE,
        format!("This account moved to the desktop app. {how}"),
    )
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::{statement, verify};
    use holo_hash::AgentPubKeyB64;
    use holochain::prelude::AgentPubKey;

    fn signed(seed: &[u8; 32], at: i64) -> (AgentPubKey, Vec<u8>) {
        let mut public = [0_u8; sodoken::sign::PUBLICKEYBYTES];
        let mut secret = sodoken::SizedLockedArray::new().expect("locked memory");
        sodoken::sign::seed_keypair(&mut public, &mut secret.lock(), seed).expect("keypair");
        let agent = AgentPubKey::from_raw_32(public.to_vec());
        let b64 = AgentPubKeyB64::from(agent.clone()).to_string();
        let mut signature = [0_u8; sodoken::sign::SIGNATUREBYTES];
        sodoken::sign::sign_detached(&mut signature, &statement(&b64, at), &secret.lock())
            .expect("sign");
        (agent, signature.to_vec())
    }

    #[test]
    fn the_phrase_key_signs_its_own_retirement() {
        let (agent, signature) = signed(&[3; 32], 1_700_000_000);
        assert!(verify(&agent, 1_700_000_000, &signature));
        // Another time, another key or a cut signature fails.
        assert!(!verify(&agent, 1_700_000_001, &signature));
        let (other, _) = signed(&[4; 32], 1_700_000_000);
        assert!(!verify(&other, 1_700_000_000, &signature));
        assert!(!verify(&agent, 1_700_000_000, &signature[..63]));
        // The desktop's `statement` (native/src/retire.rs) writes exactly these bytes.
        assert_eq!(
            statement("A", 1),
            b"social-production: retire the hosted copy of A at 1"
        );
    }
}
