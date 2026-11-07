//! `GET /index/map-markers`: web-backend's `/feeds/map-markers` (`app/routers/feeds.py`) over
//! this node's index, with web's parameters, checks and answer shape (M6 phase 4a, ADR 0082).
//! Public: the index holds public spaces only.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::{ApiError, ApiResult, AppState};
use crate::node::{utc_micros, Marker, MarkerQuery};
use sp_index::{window_bounds, Window};

const DAY: i64 = 86_400_000_000;

/// web's `_within_window` bounds: the calendar window, narrowed by `date_from`/`date_to`
/// (whole UTC days, `YYYY-MM-DD`; web's `invalid_date_filter` otherwise).
fn span(window: Window, p: &MarkerParams, now: i64) -> Result<Option<(i64, i64)>, ApiError> {
    let day = |d: &Option<String>| -> Result<Option<i64>, ApiError> {
        d.as_deref()
            .map(|d| utc_micros(d, "00:00").ok_or_else(|| refuse("invalid_date_filter")))
            .transpose()
    };
    let calendar = window_bounds(window, now, p.tz_offset.clamp(-14 * 60, 14 * 60));
    let (from, to) = (day(&p.date_from)?, day(&p.date_to)?.map(|t| t + DAY));
    if calendar.is_none() && from.is_none() && to.is_none() {
        return Ok(None);
    }
    let (mut lo, mut hi) = calendar.unwrap_or((i64::MIN, i64::MAX));
    lo = lo.max(from.unwrap_or(i64::MIN));
    hi = hi.min(to.unwrap_or(i64::MAX));
    Ok(Some((lo, hi)))
}

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
    /// `today`, `week`, `month` or `all`, in the viewer's day (the feed's `tzOffset`, minutes
    /// east of UTC, in place of web's zone name).
    window: Option<String>,
    #[serde(default, rename = "tzOffset")]
    tz_offset: i32,
    date_from: Option<String>,
    date_to: Option<String>,
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
    let window = Window::parse(p.window.as_deref().unwrap_or("all"));
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
        span: span(window, &p, now)?,
    };
    let items = state.node.index.markers(&query).await;
    Ok(Json(MarkerPage {
        total: items.len(),
        radius_km,
        center: Center {
            latitude: p.lat,
            longitude: p.lon,
        },
        window: match window {
            Window::Today => "today",
            Window::Week => "week",
            Window::Month => "month",
            Window::All => "all",
        }
        .into(),
        filter,
        items,
    }))
}
