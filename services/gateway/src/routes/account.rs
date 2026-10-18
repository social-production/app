//! Signup, restore and login (docs/plans/M1-identity.md). The browser derives the 32-byte
//! seed from the person's 24 words; we import it, bind a passkey, and host their agent.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use holo_hash::AgentPubKeyB64;
use holochain::prelude::AgentPubKey;
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::{PublicKeyCredential, RegisterPublicKeyCredential, Uuid};

use super::{client_ip, ws_url, ApiError, ApiResult, AppState};
use crate::auth::pow::Challenge;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedRequest {
    challenge: String,
    nonce: String,
    seed: Vec<u8>,
    /// Shown by the device's passkey manager.
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ceremony {
    ceremony: Uuid,
    options: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    ceremony: Uuid,
    credential: RegisterPublicKeyCredential,
    signing_key: Vec<u8>,
    restore: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    ceremony: Uuid,
    credential: PublicKeyCredential,
    signing_key: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    agent: String,
    app_ws_url: String,
    token: Vec<u8>,
    cap_secret: Vec<u8>,
    cells: Vec<CellRef>,
}

#[derive(Serialize)]
pub struct CellRef {
    dna: String,
    agent: String,
}

pub async fn pow(State(state): State<AppState>) -> Json<Challenge> {
    Json(state.pow.issue())
}

pub async fn signup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SeedRequest>,
) -> ApiResult<Ceremony> {
    let agent = import_checked(&state, &headers, &req).await?;
    if state.store.person_exists(&agent).await? {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "This recovery phrase already has an account. Sign in or restore instead.".into(),
        ));
    }
    start_passkey(&state, &agent, &req.name)
}

/// Restore on this node from the recovery phrase: same agent, new passkey for this device.
pub async fn restore(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SeedRequest>,
) -> ApiResult<Ceremony> {
    let agent = import_checked(&state, &headers, &req).await?;
    start_passkey(&state, &agent, &req.name)
}

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<SessionResponse> {
    let (agent, cred_id, passkey) = state
        .passkeys
        .finish_registration(req.ceremony, &req.credential)
        .map_err(|e| ApiError::bad_request(format!("Passkey not accepted: {e}")))?;
    state.store.upsert_person(&agent).await?;
    state.store.save_passkey(&cred_id, &agent, &passkey).await?;
    Box::pin(session(
        &state,
        &headers,
        &agent,
        &req.signing_key,
        req.restore,
    ))
    .await
}

pub async fn login_start(State(state): State<AppState>) -> ApiResult<Ceremony> {
    let (ceremony, options) = state.passkeys.start_login()?;
    Ok(Json(Ceremony {
        ceremony,
        options: serde_json::to_value(options).map_err(anyhow::Error::from)?,
    }))
}

pub async fn login_finish(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> ApiResult<SessionResponse> {
    let unknown = || {
        ApiError(
            StatusCode::UNAUTHORIZED,
            "This passkey isn't registered here. Restore with your recovery phrase.".into(),
        )
    };
    let cred_id = state
        .passkeys
        .login_credential_id(&req.credential)
        .map_err(|_| unknown())?;
    let (agent, stored) = state.store.passkey(&cred_id).await?.ok_or_else(unknown)?;
    let updated = state
        .passkeys
        .finish_login(req.ceremony, &req.credential, &stored)
        .map_err(|e| ApiError(StatusCode::UNAUTHORIZED, format!("Sign-in failed: {e}")))?;
    state.store.save_passkey(&cred_id, &agent, &updated).await?;
    Box::pin(session(&state, &headers, &agent, &req.signing_key, false)).await
}

/// Rate limit, proof-of-work, then import the seed. Returns the agent (base64).
async fn import_checked(
    state: &AppState,
    headers: &HeaderMap,
    req: &SeedRequest,
) -> Result<String, ApiError> {
    if !state.pow.verify(&req.challenge, &req.nonce) {
        return Err(ApiError::bad_request(
            "The sign-up check expired. Please try again.",
        ));
    }
    if !state.signup_limit.allow(&client_ip(headers)) {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many sign-ups from this network today.".into(),
        ));
    }
    let seed: [u8; 32] = req
        .seed
        .as_slice()
        .try_into()
        .map_err(|_| ApiError::bad_request("A recovery seed is exactly 32 bytes."))?;
    let agent = state.node.import_seed(&seed).await?;
    Ok(AgentPubKeyB64::from(agent).to_string())
}

fn start_passkey(state: &AppState, agent: &str, name: &str) -> ApiResult<Ceremony> {
    let label = if name.trim().is_empty() {
        "Social Production"
    } else {
        name.trim()
    };
    let (ceremony, options) = state.passkeys.start_registration(agent, label)?;
    Ok(Json(Ceremony { ceremony, options }))
}

/// Host the person's agent (installing it on first use) and grant this device's key.
async fn session(
    state: &AppState,
    headers: &HeaderMap,
    agent: &str,
    signing_key: &[u8],
    restore: bool,
) -> ApiResult<SessionResponse> {
    let agent_key = AgentPubKeyB64::from_b64_str(agent)
        .map(AgentPubKey::from)
        .map_err(|e| anyhow::anyhow!("stored agent key: {e}"))?;
    let signing_key = AgentPubKey::try_from_raw_39(signing_key.to_vec())
        .map_err(|_| ApiError::bad_request("signingKey must be 39 bytes"))?;
    let app = state
        .node
        .ensure_hosted(&agent_key, &state.config.happ_path, restore)
        .await?;
    let opened = state.node.open_session(&app, signing_key.clone()).await?;
    state.store.upsert_person(agent).await?;
    state
        .store
        .save_signing_key(&AgentPubKeyB64::from(signing_key).to_string(), agent)
        .await?;
    let cells = opened
        .cells
        .iter()
        .map(|c| CellRef {
            dna: holo_hash::DnaHashB64::from(c.dna_hash().clone()).to_string(),
            agent: AgentPubKeyB64::from(c.agent_pubkey().clone()).to_string(),
        })
        .collect();
    Ok(Json(SessionResponse {
        agent: agent.to_string(),
        app_ws_url: ws_url(headers),
        token: opened.token,
        cap_secret: opened.cap_secret,
        cells,
    }))
}
