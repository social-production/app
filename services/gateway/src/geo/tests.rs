use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use super::shape::{self, Location};
use super::{Geo, GeoError, Settings};

fn melbourne() -> Value {
    json!({
        "place_id": 12345, "lat": "-37.8142176", "lon": "144.9631608",
        "display_name": " Melbourne, City of Melbourne, Victoria, Australia ",
        "address": { "city": "Melbourne", "state": "Victoria", "country": "Australia" }
    })
}

#[test]
fn shapes_nominatim_like_web() {
    let found = shape::nominatim(&json!([melbourne(), { "lat": "x", "lon": "1" }]));
    assert_eq!(
        found,
        vec![Location {
            provider_place_id: Some("nominatim:12345".into()),
            display_label: "Melbourne, City of Melbourne, Victoria, Australia".into(),
            latitude: Some(-37.814_218),
            longitude: Some(144.963_161),
            region: Some("Victoria".into()),
            country: Some("Australia".into()),
            precision: "approximate",
            is_online: false,
        }]
    );
    let osm = json!({ "lat": 1, "lon": 2, "display_name": "A", "osm_type": "node", "osm_id": 9 });
    assert_eq!(
        shape::nominatim(&osm)[0].provider_place_id.as_deref(),
        Some("osm:node:9")
    );
    assert_eq!(
        shape::nominatim(&json!({ "lat": 1, "lon": 2, "display_name": "" })),
        vec![]
    );
    let long = "é".repeat(300);
    let item = json!({ "lat": 1, "lon": 2, "display_name": long });
    assert_eq!(
        shape::nominatim(&item)[0].display_label.chars().count(),
        240
    );
}

#[test]
fn shapes_ip_api_like_web() {
    let hint = shape::ip_api(&json!({
        "status": "success", "city": "Fitzroy", "regionName": "Victoria",
        "country": "Australia", "lat": -37.8, "lon": 144.98
    }))
    .expect("test setup");
    assert_eq!(hint.display_label, "Fitzroy, Victoria, Australia");
    let bare = shape::ip_api(&json!({ "status": "success", "lat": 1.234, "lon": 5.678 }))
        .expect("test setup");
    assert_eq!(bare.display_label, "1.23, 5.68");
    assert_eq!(
        shape::ip_api(&json!({ "status": "fail" })),
        Err("ip_location_unavailable")
    );
}

#[test]
fn checks_queries_and_coordinates() {
    assert_eq!(shape::query("  "), Err("geocoding_query_required"));
    assert_eq!(
        shape::query(&"a".repeat(201)),
        Err("geocoding_query_too_long")
    );
    assert_eq!(shape::query(" Fitzroy "), Ok("Fitzroy".into()));
    assert_eq!(
        shape::coordinates(91.0, 0.0),
        Err("location_coordinates_out_of_range")
    );
    assert_eq!(
        shape::coordinates(f64::NAN, 0.0),
        Err("invalid_location_coordinates")
    );
    assert_eq!(shape::viewbox("1, 2,3,4"), Some([1.0, 2.0, 3.0, 4.0]));
    assert_eq!(shape::viewbox("1,2,3"), None);
    assert_eq!(shape::viewbox("1,2,3,x"), None);
}

/// A stand-in Nominatim on localhost: `q=busy` answers 429, `q=down` 500.
async fn fake_provider(calls: Arc<AtomicUsize>) -> String {
    async fn search(
        State(calls): State<Arc<AtomicUsize>>,
        Query(q): Query<std::collections::HashMap<String, String>>,
    ) -> Result<Json<Value>, StatusCode> {
        calls.fetch_add(1, Ordering::SeqCst);
        match q.get("q").map(String::as_str) {
            Some("busy") => Err(StatusCode::TOO_MANY_REQUESTS),
            Some("down") => Err(StatusCode::INTERNAL_SERVER_ERROR),
            _ => Ok(Json(json!([melbourne()]))),
        }
    }
    let app = Router::new()
        .route("/search", get(search))
        .route("/reverse", get(|| async { Json(melbourne()) }))
        .with_state(calls);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test setup");
    let addr = listener.local_addr().expect("test setup");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("test setup") });
    format!("http://{addr}")
}

fn settings(provider_url: String) -> Settings {
    Settings {
        provider_url,
        api_key: String::new(),
        user_agent: "test".into(),
        cache_ttl: Duration::from_mins(10),
        per_ip: 2,
        window: Duration::from_mins(1),
        timeout: Duration::from_secs(5),
        ip_hint_url: "http://127.0.0.1:9/".into(),
    }
}

#[tokio::test]
async fn proxies_caches_and_maps_errors() {
    let calls = Arc::new(AtomicUsize::new(0));
    let url = fake_provider(calls.clone()).await;
    let geo =
        Geo::with_client(settings(url), reqwest::Client::builder().no_proxy()).expect("test setup");
    let found = geo
        .search("Melbourne", 5, None, None)
        .await
        .expect("test setup");
    assert_eq!(found[0].region.as_deref(), Some("Victoria"));
    geo.search(" melbourne ", 5, None, None)
        .await
        .expect("test setup");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "the second search is a cache hit"
    );
    let busy = geo.search("busy", 5, None, None).await;
    assert_eq!(
        busy,
        Err(GeoError(
            StatusCode::TOO_MANY_REQUESTS,
            "geocoding_provider_rate_limited"
        ))
    );
    let down = geo.search("down", 5, None, None).await;
    assert_eq!(
        down,
        Err(GeoError(
            StatusCode::SERVICE_UNAVAILABLE,
            "geocoding_provider_unavailable"
        ))
    );
    assert_eq!(
        geo.reverse(-37.81, 144.96).await.expect("test setup").len(),
        1
    );
    assert!(geo.check_limit("1.2.3.4").is_ok() && geo.check_limit("1.2.3.4").is_ok());
    assert_eq!(
        geo.check_limit("1.2.3.4"),
        Err(GeoError(
            StatusCode::TOO_MANY_REQUESTS,
            "geocoding_rate_limit_exceeded"
        ))
    );
    let local = geo.ip_hint("127.0.0.1").await;
    assert_eq!(
        local,
        Err(GeoError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "ip_location_unavailable"
        ))
    );
}
