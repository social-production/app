//! HTTP API of the gateway. Every successful sign-in ends in a [`account::SessionResponse`]
//! the browser uses to open its app websocket (docs/IDENTITY.md).

pub mod account;
pub mod index;
pub mod passkey;
pub mod public;
pub mod session;
pub mod ws_proxy;

use std::sync::Arc;

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{extract::State, Json, Router};
use tower_http::cors::CorsLayer;

use crate::auth::{passkeys::Passkeys, pow::Pow, ratelimit::SignupLimit};
use crate::{config::Config, node::Node, store::Store};

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<Config>,
    pub node: Node,
    pub store: Store,
    pub passkeys: Arc<Passkeys>,
    pub pow: Arc<Pow>,
    pub signup_limit: Arc<SignupLimit>,
    pub signin_limit: Arc<SignupLimit>,
    /// Guest reads (`/public/read`) per IP per day.
    pub read_limit: Arc<SignupLimit>,
    /// When this process started (`/readyz` uptime).
    pub started: std::time::Instant,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(readyz))
        .route("/account/pow", post(account::pow))
        .route("/account/open", post(account::open))
        .route("/account/passkey/start", post(passkey::add_start))
        .route("/account/passkey/finish", post(passkey::add_finish))
        .route("/account/login/start", post(account::login_start))
        .route("/account/login/finish", post(account::login_finish))
        .route("/session/resume", post(session::resume))
        .route("/session/cells", post(session::cells))
        .route("/index/projects", get(index::projects))
        .route("/index/people", get(index::people))
        .route("/public/read", post(public::read))
        .route("/ws", get(ws_proxy::upgrade))
        // The UI is served from another origin (ui service, dev server).
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let ready = state.node.is_ready().await;
    let status = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    // Uptime and resident memory let a session without Railway access see restarts and growth.
    let body = serde_json::json!({
        "conductor": ready,
        "uptimeSecs": state.started.elapsed().as_secs(),
        "rssMb": resident_mb(),
    });
    (status, Json(body))
}

/// This process's resident memory in MB (Linux), from `/proc/self/status`.
fn resident_mb() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("VmRSS:"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / 1024)
}

/// The `/ws` URL on whatever host the browser used; Railway sets X-Forwarded-Proto.
pub fn ws_url(headers: &HeaderMap) -> String {
    let host = header(headers, "host").unwrap_or("localhost:8080");
    let secure = header(headers, "x-forwarded-proto") == Some("https");
    format!("{}://{host}/ws", if secure { "wss" } else { "ws" })
}

/// Client IP as seen by Railway's proxy (first X-Forwarded-For hop).
pub fn client_ip(headers: &HeaderMap) -> String {
    header(headers, "x-forwarded-for")
        .and_then(|v| v.split(',').next())
        .map_or_else(|| "direct".to_string(), |ip| ip.trim().to_string())
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

/// An error the browser can show: a status and one readable sentence.
#[derive(Debug)]
pub struct ApiError(pub StatusCode, pub String);

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, message.into())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        tracing::warn!(error = %e, "request failed");
        Self(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({ "error": self.1 }))).into_response()
    }
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;
