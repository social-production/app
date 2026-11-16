//! Web Push subscriptions (ADR 0108). A signed-in browser that switched push on sends its
//! push subscription here, proving it is the device that signed in with its device secret
//! (as `/session/cells` does). The pushes themselves start from signals (`::push::Pusher`).

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use holo_hash::AgentPubKeyB64;
use holochain::prelude::AgentPubKey;
use serde::{Deserialize, Serialize};

use super::{ApiError, ApiResult, AppState};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    public_key: String,
}

/// The key a browser subscribes with (`applicationServerKey`).
pub async fn key(State(state): State<AppState>) -> ApiResult<Key> {
    Ok(Json(Key {
        public_key: state.pusher.public_key(),
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeRequest {
    signing_key: Vec<u8>,
    device_secret: Vec<u8>,
    subscription: ::push::Subscription,
    /// Notification categories switched off on this device.
    #[serde(default)]
    muted: Vec<String>,
    /// "Show what it's about": off sends a push without the card's words.
    #[serde(default = "yes")]
    details: bool,
}

fn yes() -> bool {
    true
}

#[derive(Serialize)]
pub struct Done {
    ok: bool,
}

/// Switch push on in this browser, or change its choices.
pub async fn subscribe(
    State(state): State<AppState>,
    Json(req): Json<SubscribeRequest>,
) -> ApiResult<Done> {
    let agent = agent_of(&state, req.signing_key, &req.device_secret).await?;
    if req.muted.len() > 20 {
        return Err(ApiError::bad_request("too many categories"));
    }
    let saved = ::push::Saved {
        subscription: req.subscription,
        muted: req.muted,
        details: req.details,
    };
    state
        .pusher
        .subscribe(&agent, &saved)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    Ok(Json(Done { ok: true }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsubscribeRequest {
    signing_key: Vec<u8>,
    device_secret: Vec<u8>,
    endpoint: String,
}

/// Switch push off in this browser (also on sign-out).
pub async fn unsubscribe(
    State(state): State<AppState>,
    Json(req): Json<UnsubscribeRequest>,
) -> ApiResult<Done> {
    let agent = agent_of(&state, req.signing_key, &req.device_secret).await?;
    let ok = state.pusher.unsubscribe(&agent, &req.endpoint).await?;
    Ok(Json(Done { ok }))
}

/// The hosted agent behind a signed-in device.
async fn agent_of(
    state: &AppState,
    signing_key: Vec<u8>,
    device_secret: &[u8],
) -> Result<String, ApiError> {
    let gone = || ApiError(StatusCode::UNAUTHORIZED, "Please sign in again.".into());
    let key = AgentPubKey::try_from_raw_39(signing_key).map_err(|_| gone())?;
    let key_b64 = AgentPubKeyB64::from(key).to_string();
    state
        .store
        .agent_for_device(&key_b64, device_secret)
        .await?
        .ok_or_else(gone)
}
