use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::CorsLayer;

use crate::{conductor::Conductor, config::Config, session, ws_proxy};

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub conductor: Conductor,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/session", post(session::open))
        .route("/ws", get(ws_proxy::upgrade))
        // The UI may be served from another origin (ui service, Tauri, dev server).
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}

/// Ready once the conductor answers admin calls and the demo app is installed.
async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<Ready>) {
    let conductor = state.conductor.is_ready().await;
    let status = if conductor {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(Ready { conductor }))
}

#[derive(serde::Serialize)]
struct Ready {
    conductor: bool,
}
