//! `GET /locations/search`, `/locations/reverse`, `/locations/ip-hint`: web-backend's routes
//! of the same names (`app/routers/locations.py`), same parameters and `{ items: [...] }`
//! answers, so the place picker ports unchanged (M6 phase 2, ADR 0079). Public, as on web.

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::{client_ip, ApiError, ApiResult, AppState};
use axum::http::StatusCode;
use geo::{GeoError, Location};

#[derive(Serialize)]
pub struct Items {
    items: Vec<Location>,
}

impl From<GeoError> for ApiError {
    fn from(GeoError(status, code): GeoError) -> Self {
        let status = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
        Self(status, code.into())
    }
}

#[derive(Deserialize)]
pub struct SearchParams {
    #[serde(default)]
    q: String,
    limit: Option<usize>,
    country_codes: Option<String>,
    viewbox: Option<String>,
}

pub async fn search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(p): Query<SearchParams>,
) -> ApiResult<Items> {
    let limit = p.limit.unwrap_or(5);
    geo::check_search(
        &p.q,
        limit,
        p.country_codes.as_deref(),
        p.viewbox.as_deref(),
    )?;
    state.geo.check_limit(&client_ip(&headers))?;
    let viewbox = p.viewbox.as_deref().and_then(geo::shape::viewbox);
    let items = state
        .geo
        .search(&p.q, limit, p.country_codes.as_deref(), viewbox)
        .await?;
    Ok(Json(Items { items }))
}

#[derive(Deserialize)]
pub struct ReverseParams {
    lat: f64,
    lon: f64,
}

pub async fn reverse(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(p): Query<ReverseParams>,
) -> ApiResult<Items> {
    state.geo.check_limit(&client_ip(&headers))?;
    let items = state.geo.reverse(p.lat, p.lon).await?;
    Ok(Json(Items { items }))
}

pub async fn ip_hint(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Items> {
    let hint = state.geo.ip_hint(&client_ip(&headers)).await?;
    Ok(Json(Items { items: vec![hint] }))
}
