//! Signing up, restoring and signing in (docs/IDENTITY.md). The recovery phrase is always
//! enough: the browser derives the 32-byte seed from the 24 words and we open a session for
//! that agent. Passkeys are an optional shortcut added afterwards (`routes/passkey.rs`).

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use holo_hash::AgentPubKeyB64;
use holochain::prelude::AgentPubKey;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::{PublicKeyCredential, Uuid};

use super::{client_ip, ws_url, ApiError, ApiResult, AppState};
use crate::auth::pow::Challenge;

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OpenMode {
    /// New account: the phrase must not have one on this node yet.
    Signup,
    /// Existing identity on this node or another: host it here.
    Restore,
    /// Existing account on this node.
    Signin,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenRequest {
    mode: OpenMode,
    /// Proof-of-work, required for signup and restore (they create hosted agents).
    challenge: Option<String>,
    nonce: Option<String>,
    seed: Vec<u8>,
    signing_key: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ceremony {
    pub ceremony: Uuid,
    pub options: serde_json::Value,
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
    /// Proves this device later (e.g. to add a passkey). Kept by the browser only.
    device_secret: Vec<u8>,
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

/// Open a session from the recovery phrase.
pub async fn open(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<OpenRequest>,
) -> ApiResult<SessionResponse> {
    check_limits(&state, &headers, &req)?;
    let seed: [u8; 32] = req
        .seed
        .as_slice()
        .try_into()
        .map_err(|_| ApiError::bad_request("A recovery seed is exactly 32 bytes."))?;
    let agent = AgentPubKeyB64::from(state.node.import_seed(&seed).await?).to_string();
    let exists = state.store.person_exists(&agent).await?;
    match (req.mode, exists) {
        (OpenMode::Signup, true) => Err(ApiError(
            StatusCode::CONFLICT,
            "This recovery phrase already has an account. Sign in instead.".into(),
        )),
        (OpenMode::Signin, false) => Err(ApiError(
            StatusCode::NOT_FOUND,
            "No account with this phrase on this site. Check the words, or use Restore.".into(),
        )),
        _ => {
            let restore = req.mode == OpenMode::Restore && !exists;
            Box::pin(session(&state, &headers, &agent, &req.signing_key, restore)).await
        }
    }
}

/// Sign-in is rate limited; signup and restore (which create hosted agents) also need the
/// proof-of-work and the signup limit.
fn check_limits(state: &AppState, headers: &HeaderMap, req: &OpenRequest) -> Result<(), ApiError> {
    let ip = client_ip(headers);
    if req.mode == OpenMode::Signin {
        if !state.signin_limit.allow(&ip) {
            return Err(too_many(
                "Too many sign-in attempts from this network today.",
            ));
        }
        return Ok(());
    }
    let solved = matches!(
        (&req.challenge, &req.nonce),
        (Some(c), Some(n)) if state.pow.verify(c, n)
    );
    if !solved {
        return Err(ApiError::bad_request(
            "The sign-up check expired. Please try again.",
        ));
    }
    if !state.signup_limit.allow(&ip) {
        return Err(too_many("Too many sign-ups from this network today."));
    }
    Ok(())
}

pub async fn login_start(State(state): State<AppState>) -> ApiResult<Ceremony> {
    let (ceremony, options) = state.passkeys.start_login()?;
    Ok(Json(Ceremony { ceremony, options }))
}

pub async fn login_finish(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> ApiResult<SessionResponse> {
    let unknown = || {
        ApiError(
            StatusCode::UNAUTHORIZED,
            "This passkey isn't registered here. Sign in with your recovery phrase.".into(),
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

fn too_many(message: &str) -> ApiError {
    ApiError(StatusCode::TOO_MANY_REQUESTS, message.into())
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
    let opened = Box::pin(state.node.open_session(&app, signing_key.clone())).await?;
    let mut device_secret = vec![0_u8; 32];
    rand::thread_rng().fill_bytes(&mut device_secret);
    state.store.upsert_person(agent).await?;
    let key = AgentPubKeyB64::from(signing_key).to_string();
    state
        .store
        .save_signing_key(&key, agent, &device_secret)
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
        device_secret,
        cells,
    }))
}
