//! `GET /index/projects`: the project feed with web's sorts and windows, plus search
//! (docs/plans/M2-projects.md). Public, read-only, served from this node's index.

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use sp_index::{FeedQuery, Sort, Window};

use super::{ApiResult, AppState};
use crate::node::{PersonCard, ProjectCard};

const PAGE_MAX: usize = 50;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedParams {
    #[serde(default)]
    sort: String,
    #[serde(default)]
    window: String,
    #[serde(default)]
    q: String,
    #[serde(default)]
    offset: usize,
    limit: Option<usize>,
    /// The viewer's UTC offset in minutes east (JS: `-new Date().getTimezoneOffset()`).
    #[serde(default)]
    tz_offset: i32,
}

#[derive(Serialize)]
pub struct FeedPage {
    items: Vec<ProjectCard>,
    total: usize,
}

pub async fn projects(
    State(state): State<AppState>,
    Query(p): Query<FeedParams>,
) -> ApiResult<FeedPage> {
    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros(),
    )
    .unwrap_or(i64::MAX);
    let query = FeedQuery {
        sort: Sort::parse(&p.sort),
        window: Window::parse(&p.window),
        search: &p.q,
        now,
        utc_offset_minutes: p.tz_offset.clamp(-14 * 60, 14 * 60),
    };
    let limit = p.limit.unwrap_or(20).min(PAGE_MAX);
    let (items, total) = state.node.index.page(&query, p.offset, limit).await;
    Ok(Json(FeedPage { items, total }))
}

#[derive(Deserialize)]
pub struct PeopleParams {
    #[serde(default)]
    q: String,
    limit: Option<usize>,
}

/// `GET /index/people?q=`: people whose name or bio matches, for web's Search page.
pub async fn people(
    State(state): State<AppState>,
    Query(p): Query<PeopleParams>,
) -> ApiResult<Vec<PersonCard>> {
    let limit = p.limit.unwrap_or(20).min(PAGE_MAX);
    Ok(Json(state.node.people.search(&p.q, limit).await))
}
