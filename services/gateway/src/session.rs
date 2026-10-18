//! `POST /session`: hand a browser what it needs to call zomes on its hosted cells.

use axum::{extract::State, http::HeaderMap, http::StatusCode, Json};
use holo_hash::{AgentPubKeyB64, DnaHashB64};
use holochain_client::AgentPubKey;
use serde::{Deserialize, Serialize};

use crate::routes::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequest {
    /// The browser's own zome-call signing key: the 39 raw bytes of an `AgentPubKey`.
    /// Raw, not base64: client-generated keys carry random location bytes, so they
    /// fail the base64 form's checksum, and the conductor matches all 39 bytes anyway.
    signing_key: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
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

pub async fn open(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SessionRequest>,
) -> Result<Json<SessionResponse>, (StatusCode, String)> {
    let signing_key = AgentPubKey::try_from_raw_39(request.signing_key)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("signingKey: {e}")))?;
    let session = state
        .conductor
        .open_session(signing_key)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, e.to_string()))?;

    let cells = session
        .cells
        .iter()
        .map(|cell| CellRef {
            dna: DnaHashB64::from(cell.dna_hash().clone()).to_string(),
            agent: AgentPubKeyB64::from(cell.agent_pubkey().clone()).to_string(),
        })
        .collect();
    Ok(Json(SessionResponse {
        app_ws_url: ws_url(&headers),
        token: session.token,
        cap_secret: session.cap_secret,
        cells,
    }))
}

/// The `/ws` URL on whatever host the browser used; Railway sets X-Forwarded-Proto.
fn ws_url(headers: &HeaderMap) -> String {
    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:8080");
    let secure = headers
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .is_some_and(|proto| proto == "https");
    format!("{}://{host}/ws", if secure { "wss" } else { "ws" })
}
