//! `GET /index/region`: web-backend's `/feeds/region` (`app/routers/feeds.py`) over this
//! node's index, with web's parameters, checks and echoes (M6 phase 4c, ADR 0084). Items are
//! the feed's camelCase cards (plus `distanceKm`, `isOnline`) and web's project-activity
//! items, as `/index/projects` sends them. Public: the index holds public spaces only.

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use sp_index::{Sort, Window};

use super::map::{checked_area, Center};
use super::{ApiResult, AppState};
use crate::node::{RegionItem, RegionQuery};

/// web's `limit` bound (`le=100`).
const PAGE_MAX: usize = 100;

#[derive(Deserialize)]
pub struct RegionParams {
    lat: f64,
    lon: f64,
    radius_km: Option<u32>,
    #[serde(default)]
    sort: String,
    #[serde(default)]
    window: String,
    #[serde(default)]
    filter: String,
    #[serde(default)]
    include_online: bool,
    /// Minutes east of UTC, in place of web's zone name (as `/index/projects`).
    #[serde(default, rename = "tzOffset")]
    tz_offset: i32,
    limit: Option<usize>,
    #[serde(default)]
    offset: usize,
}

/// web's region `FeedResponse`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionPage {
    items: Vec<RegionItem>,
    total: usize,
    sort: &'static str,
    window: &'static str,
    filter: String,
    limit: usize,
    offset: usize,
    radius_km: u32,
    center: Center,
    include_online: bool,
}

pub async fn region(
    State(state): State<AppState>,
    Query(p): Query<RegionParams>,
) -> ApiResult<RegionPage> {
    let radius_km = checked_area(p.lat, p.lon, p.radius_km)?;
    // web's `normalize_filter`.
    let filter = match p.filter.trim().to_lowercase().as_str() {
        f @ ("projects" | "events" | "help_requests" | "threads") => f.to_owned(),
        _ => "all".to_owned(),
    };
    let (sort, window) = (Sort::parse(&p.sort), Window::parse(&p.window));
    let limit = p.limit.unwrap_or(20).clamp(1, PAGE_MAX);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_micros()).unwrap_or(i64::MAX));
    let query = RegionQuery {
        center: (p.lat, p.lon),
        radius_km: f64::from(radius_km),
        sort,
        window,
        filter: &filter,
        include_online: p.include_online,
        utc_offset_minutes: p.tz_offset.clamp(-14 * 60, 14 * 60),
        limit,
        offset: p.offset,
        now,
    };
    let (items, total) = state.node.index.region(&query).await;
    Ok(Json(RegionPage {
        items,
        total,
        sort: match sort {
            Sort::Trending => "trending",
            Sort::Recent => "recent",
            Sort::Top => "top",
            Sort::Oldest => "oldest",
        },
        window: match window {
            Window::Today => "today",
            Window::Week => "week",
            Window::Month => "month",
            Window::All => "all",
        },
        filter,
        limit,
        offset: p.offset,
        radius_km,
        center: Center {
            latitude: p.lat,
            longitude: p.lon,
        },
        include_online: p.include_online,
    }))
}
