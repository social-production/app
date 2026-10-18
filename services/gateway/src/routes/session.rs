//! Reconnect a device that already signed in: its signing key was granted access then,
//! and it keeps that key, so a new app-websocket token is all it needs. Knowing a public
//! signing key alone is useless: every zome call must be signed with its private half.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use holo_hash::{AgentPubKeyB64, HoloHashB64};
use holochain::prelude::AgentPubKey;
use serde::{Deserialize, Serialize};

use super::{ws_url, ApiError, ApiResult, AppState};
use crate::node::app_id;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeRequest {
    signing_key: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resumed {
    agent: String,
    app_ws_url: String,
    token: Vec<u8>,
}

pub async fn resume(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ResumeRequest>,
) -> ApiResult<Resumed> {
    let gone = || ApiError(StatusCode::UNAUTHORIZED, "Please sign in again.".into());
    let signing_key = AgentPubKey::try_from_raw_39(req.signing_key).map_err(|_| gone())?;
    let key_b64 = AgentPubKeyB64::from(signing_key).to_string();
    let agent = state
        .store
        .agent_for_signing_key(&key_b64)
        .await?
        .ok_or_else(gone)?;
    let agent_key: AgentPubKey = HoloHashB64::from_b64_str(&agent)
        .map(Into::into)
        .map_err(|e| anyhow::anyhow!("stored agent key: {e}"))?;
    // After a network reset (ADR 0028) the person's app for this hApp version doesn't exist
    // yet: send them to sign in (which installs it) instead of issuing a useless token.
    let id = app_id(&agent_key, &state.node.happ_version);
    if state.node.find_app(&id).await?.is_none() {
        return Err(gone());
    }
    let token = state.node.app_token(&id).await.map_err(|_| gone())?;
    state.store.upsert_person(&agent).await?;
    Ok(Json(Resumed {
        agent,
        app_ws_url: ws_url(&headers),
        token,
    }))
}
