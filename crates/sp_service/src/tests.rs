use super::*;

#[test]
fn checks_requests() {
    assert!(check_request("p", "Bikes", "Fix my bike", "Flat tyre", None).is_ok());
    assert!(check_request("p", "Bikes", "Fix", "Flat", Some((1, 2))).is_ok());
    assert!(check_request("", "Bikes", "Fix", "Flat", None).is_err());
    assert!(check_request("p", "Bikes", " ", "Flat", None).is_err());
    assert!(check_request("p", "Bikes", &"x".repeat(TITLE_MAX + 1), "Flat", None).is_err());
    assert!(check_request("p", "Bikes", "Fix", "", None).is_err());
    assert!(check_request("p", "Bikes", "Fix", "Flat", Some((2, 2))).is_err());
    assert!(check_request("p", "", "Fix", "Flat", None).is_err());
}

#[test]
fn checks_statuses() {
    assert!(check_status("accepted", "").is_ok());
    assert!(check_status("declined", "Busy that week").is_ok());
    assert!(check_status("cancelled", "").is_ok());
    assert!(check_status("open", "").is_err());
    assert!(check_status("accepted", "why").is_err());
    assert!(check_status("declined", &"x".repeat(REASON_MAX + 1)).is_err());
}

#[test]
fn requester_cancels_and_creator_decides() {
    assert!(may_decide("cancelled", true));
    assert!(!may_decide("cancelled", false));
    assert!(may_decide("accepted", false));
    assert!(!may_decide("accepted", true));
    assert!(!may_decide("declined", true));
}

#[test]
fn first_allowed_decision_settles() {
    assert_eq!(replay([]), State::Pending);
    // The requester can't accept their own request; the creator's decline then counts.
    let state = replay([("accepted", "", true), ("declined", "Away", false)]);
    assert_eq!(
        state,
        State::Declined {
            reason: "Away".into()
        }
    );
    assert_eq!(
        replay([("cancelled", "", true), ("accepted", "", false)]),
        State::Cancelled
    );
    assert_eq!(replay([("accepted", "", false)]).id(), "accepted");
}

#[test]
fn chat_lines_match_web() {
    assert_eq!(
        request_message("Fix", " Flat tyre ", None, "/projects/p?request=r"),
        "Request \"Fix\" · Pending\n\nFlat tyre\n\nView request: /projects/p?request=r"
    );
    assert_eq!(
        request_message("Fix", "Flat", Some(("a", "b")), "h"),
        "Request \"Fix\" · Pending\n\nFlat\n\nWhen: a – b\n\nView request: h"
    );
    assert_eq!(
        status_message("Fix", "declined", "Away", "h"),
        "Request \"Fix\" · Declined\n\nReason: Away\n\nView request: h"
    );
    assert_eq!(
        status_message("Fix", "accepted", "", "h"),
        "Request \"Fix\" · Accepted\n\nView request: h"
    );
}

#[test]
fn settings_defaults_and_checks() {
    assert_eq!(
        Settings::default_for("personal-service"),
        Settings {
            enabled: true,
            mode: "both".into(),
            off_schedule: true
        }
    );
    assert!(!Settings::default_for("collective-service").enabled);
    assert!(!Settings::created_with("calendar").off_schedule);
    assert!(check_settings("personal-service", "direct", "").is_ok());
    assert!(check_settings("collective-service", "direct", "").is_err());
    assert!(check_settings("collective-service", "direct", "Busy season").is_ok());
    assert!(check_settings("productive", "both", "x").is_err());
    assert!(check_settings("personal-service", "weekly", "").is_err());
}

#[test]
fn admits_like_web() {
    let off = Settings {
        enabled: false,
        mode: "both".into(),
        off_schedule: true,
    };
    assert_eq!(admits(&off, false), Err("Not taking requests.".to_string()));
    let calendar = Settings::created_with("calendar");
    assert!(admits(&calendar, false).is_err());
    assert!(admits(&calendar, true).is_ok());
    let direct = Settings::created_with("direct");
    assert!(admits(&direct, false).is_ok());
    assert!(admits(&direct, true).is_err());
    assert!(admits(&Settings::created_with("both"), true).is_ok());
}
