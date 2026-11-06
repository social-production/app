use super::*;

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn physical(lat: f64, lon: f64, precision: &str) -> PlaceRef<'static> {
    PlaceRef {
        label: "Fitzroy, Melbourne",
        coordinates: Some((lat, lon)),
        region: Some("Victoria"),
        country: Some("Australia"),
        provider_id: None,
        precision: if precision == "exact" {
            "exact"
        } else {
            "approximate"
        },
        online: false,
    }
}

#[test]
fn rounds_to_the_precision() {
    assert!(near(round(-37.798_765, Precision::Approximate), -37.8));
    assert!(near(round(144.978_123_456, Precision::Exact), 144.978_123));
}

#[test]
fn accepts_rounded_physical_places() {
    assert_eq!(check(&physical(-37.8, 144.98, "approximate")), Ok(()));
    assert_eq!(check(&physical(-37.798_765, 144.978_123, "exact")), Ok(()));
}

#[test]
fn refuses_extra_decimals_and_out_of_range() {
    assert!(check(&physical(-37.798, 144.98, "approximate")).is_err());
    assert!(check(&physical(91.0, 0.0, "approximate")).is_err());
    assert!(check(&physical(0.0, -181.0, "exact")).is_err());
}

#[test]
fn online_places_have_no_coordinates() {
    let mut p = physical(-37.8, 144.98, "approximate");
    p.online = true;
    assert!(check(&p).is_err());
    p.coordinates = None;
    assert_eq!(check(&p), Ok(()));
    p.online = false;
    assert!(check(&p).is_err());
}

#[test]
fn labels_and_areas_have_limits() {
    let long = "x".repeat(MAX_LABEL + 1);
    let mut p = physical(-37.8, 144.98, "approximate");
    p.label = &long;
    assert!(check(&p).is_err());
    p.label = "  ";
    assert!(check(&p).is_err());
    p.label = "Fitzroy";
    p.region = Some("");
    assert!(check(&p).is_err());
    p.region = None;
    p.precision = "rough";
    assert!(check(&p).is_err());
}

#[test]
fn distance_matches_web() {
    // Melbourne CBD to Sydney CBD is about 714 km.
    let d = distance_km((-37.8136, 144.9631), (-33.8688, 151.2093));
    assert!((d - 713.0).abs() < 5.0, "{d}");
    assert!(near(distance_km((1.0, 2.0), (1.0, 2.0)), 0.0));
}
