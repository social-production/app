//! `GET /locations/search`, `/locations/reverse`, `/locations/ip-hint`: web-backend's routes
//! of the same names (`app/routers/locations.py`), same parameters and `{ items: [...] }`
//! answers, so the place picker ports unchanged (M6 phase 2, ADR 0079). Public, as on web.

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::{client_ip, ApiError, ApiResult, AppState};
use crate::geo::{shape, GeoError, Location};

#[derive(Serialize)]
pub struct Items {
    items: Vec<Location>,
}

impl From<GeoError> for ApiError {
    fn from(GeoError(status, code): GeoError) -> Self {
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
    // web's FastAPI checks: q 1–200 characters, limit 1–10 (5 by default), short extras.
    let limit = p.limit.unwrap_or(5);
    let too_long = |v: &Option<String>, max: usize| v.as_ref().is_some_and(|s| s.len() > max);
    if p.q.is_empty()
        || p.q.chars().count() > shape::MAX_QUERY
        || !(1..=shape::MAX_RESULTS).contains(&limit)
        || too_long(&p.country_codes, 40)
        || too_long(&p.viewbox, 80)
    {
        return Err(ApiError(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_location_search".into(),
        ));
    }
    state.geo.check_limit(&client_ip(&headers))?;
    let viewbox = p.viewbox.as_deref().and_then(shape::viewbox);
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
