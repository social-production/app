//! `POST /public/read`: signed-out people read project pages (node/public.rs). Answers
//! msgpack, as a zome call would; refused calls are 403, data this node lacks is 404.

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use super::{client_ip, ApiError, AppState};
use crate::node::{PublicRead, Refused};

pub async fn read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(read): Json<PublicRead>,
) -> Result<Response, ApiError> {
    if !state.read_limit.allow(&client_ip(&headers)) {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many reads from this address today. Sign in to keep going.".into(),
        ));
    }
    match Box::pin(state.node.public_read(read)).await? {
        Ok(bytes) => Ok(([(header::CONTENT_TYPE, "application/msgpack")], bytes).into_response()),
        Err(Refused::NotAllowed) => Err(ApiError(
            StatusCode::FORBIDDEN,
            "Sign in to continue.".into(),
        )),
        Err(Refused::NotHere) => Err(ApiError(
            StatusCode::NOT_FOUND,
            "This project isn't on this node yet.".into(),
        )),
    }
}
