//! Add a passkey to an account from a device that is already signed in, so the person can
//! skip typing their words next time. Requires the device secret issued at sign-in: knowing
//! someone's public signing key is not enough to attach a passkey to their account.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use holo_hash::AgentPubKeyB64;
use holochain::prelude::AgentPubKey;
use serde::Deserialize;
use webauthn_rs::prelude::{RegisterPublicKeyCredential, Uuid};

use super::account::Ceremony;
use super::{ApiError, ApiResult, AppState};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceProof {
    signing_key: Vec<u8>,
    device_secret: Vec<u8>,
    /// Shown by the device's passkey manager.
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinishRequest {
    ceremony: Uuid,
    credential: RegisterPublicKeyCredential,
}

pub async fn add_start(
    State(state): State<AppState>,
    Json(req): Json<DeviceProof>,
) -> ApiResult<Ceremony> {
    let denied = || {
        ApiError(
            StatusCode::UNAUTHORIZED,
            "Please sign in again first.".into(),
        )
    };
    let key = AgentPubKey::try_from_raw_39(req.signing_key).map_err(|_| denied())?;
    let key = AgentPubKeyB64::from(key).to_string();
    let agent = state
        .store
        .agent_for_device(&key, &req.device_secret)
        .await?
        .ok_or_else(denied)?;
    let label = if req.name.trim().is_empty() {
        "Social Production"
    } else {
        req.name.trim()
    };
    let (ceremony, options) = state.passkeys.start_registration(&agent, label)?;
    Ok(Json(Ceremony { ceremony, options }))
}

pub async fn add_finish(
    State(state): State<AppState>,
    Json(req): Json<FinishRequest>,
) -> ApiResult<serde_json::Value> {
    let (agent, cred_id, passkey) = state
        .passkeys
        .finish_registration(req.ceremony, &req.credential)
        .map_err(|e| ApiError::bad_request(format!("Passkey not accepted: {e}")))?;
    state.store.save_passkey(&cred_id, &agent, &passkey).await?;
    Ok(Json(serde_json::json!({ "saved": true })))
}
