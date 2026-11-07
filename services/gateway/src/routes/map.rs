//! `GET /index/map-markers`: web-backend's `/feeds/map-markers` (`app/routers/feeds.py`) over
//! this node's index, with web's parameters, checks and answer shape (M6 phase 4a, ADR 0082).
//! Public: the index holds public spaces only.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::{ApiError, ApiResult, AppState};
use crate::node::{Marker, MarkerQuery};

/// web's `DEFAULT_RADIUS_KM` and `MAX_RADIUS_KM`.
const RADIUS_DEFAULT: u32 = 25;
const RADIUS_MAX: u32 = 20_000;

#[derive(Deserialize)]
pub struct MarkerParams {
    lat: f64,
    lon: f64,
    radius_km: Option<u32>,
    filter: Option<String>,
    upcoming_only: Option<bool>,
    limit: Option<usize>,
}

/// web's `MapMarkersResponse`.
#[derive(Serialize)]
pub struct MarkerPage {
    total: usize,
    radius_km: u32,
    center: Center,
    window: String,
    filter: String,
    items: Vec<Marker>,
}

#[derive(Serialize)]
pub struct Center {
    latitude: f64,
    longitude: f64,
}

fn refuse(code: &str) -> ApiError {
    ApiError(StatusCode::UNPROCESSABLE_ENTITY, code.into())
}

pub async fn map_markers(
    State(state): State<AppState>,
    Query(p): Query<MarkerParams>,
) -> ApiResult<MarkerPage> {
    // web's `validate_region_center` and `normalize_radius_km`.
    if !(-90.0..=90.0).contains(&p.lat) || !(-180.0..=180.0).contains(&p.lon) {
        return Err(refuse("region_coordinates_out_of_range"));
    }
    let radius_km = p.radius_km.unwrap_or(RADIUS_DEFAULT);
    if !(1..=RADIUS_MAX).contains(&radius_km) {
        return Err(refuse("invalid_region_radius"));
    }
    let filter = match p.filter.as_deref() {
        Some(f @ ("projects" | "events" | "help_requests")) => f.to_owned(),
        _ => "all".to_owned(),
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_micros()).unwrap_or(i64::MAX));
    let query = MarkerQuery {
        center: (p.lat, p.lon),
        radius_km: f64::from(radius_km),
        filter: &filter,
        upcoming_only: p.upcoming_only.unwrap_or(true),
        limit: p.limit.unwrap_or(200).clamp(1, 500),
        now,
    };
    let items = state.node.index.markers(&query).await;
    Ok(Json(MarkerPage {
        total: items.len(),
        radius_km,
        center: Center {
            latitude: p.lat,
            longitude: p.lon,
        },
        // web's calendar windows and dates come with the Region feed (phase 4b).
        window: "all".into(),
        filter,
        items,
    }))
}
