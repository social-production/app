//! Activities and roles in a project (web's `project_activities`, `_roles`, `_assignments`,
//! `_ratings`; ADR 0056). Shared by the project integrity zome (validation) and its
//! coordinator (input checks and the view). Not in `sp_core`: code there changes every DNA's
//! hash.

mod fill;

pub use fill::{fill, Taken};

use sp_core::limits::{optional_text_ok, text_ok, LONG_TEXT_MAX};

/// web-backend `ProjectActivityCreateRequest.title`.
pub const TITLE_MAX: usize = 200;
/// web-backend `location_label`.
pub const PLACE_MAX: usize = 160;
/// web-backend `meeting_url`.
pub const MEETING_URL_MAX: usize = 500;
/// web-backend `ProjectActivityRoleRequirementIn.label`.
pub const ROLE_LABEL_MAX: usize = 100;
/// web has no cap; a bound keeps an entry small.
pub const ROLES_MAX: usize = 20;
/// web has no cap on a role's counts; this keeps them sane.
pub const COUNT_MAX: u32 = 1_000;
/// web-backend `ProjectActivityRatingRequest.comment`.
pub const COMMENT_MAX: usize = 2_000;

/// One role an activity needs (web's role requirement).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleSpec<'a> {
    pub label: &'a str,
    pub required: u32,
    pub maximum: Option<u32>,
}

/// What an activity says about itself, borrowed from the entry.
#[derive(Clone, Debug)]
pub struct ActivitySpec<'a> {
    pub title: &'a str,
    /// Holochain timestamps (microseconds).
    pub starts_at: i64,
    pub ends_at: i64,
    pub online: bool,
    pub place: &'a str,
    pub meeting_url: Option<&'a str>,
    pub note: &'a str,
    pub roles: Vec<RoleSpec<'a>>,
}

/// web-backend's request rules for a new activity (the phase and "starts in the future" are
/// the coordinator's: validation has no "now").
pub fn check_activity(a: &ActivitySpec<'_>) -> Result<(), String> {
    if !text_ok(a.title, TITLE_MAX) {
        return Err(format!("an activity's title is 1–{TITLE_MAX} characters"));
    }
    if a.ends_at <= a.starts_at {
        return Err("an activity ends after it starts".into());
    }
    if !text_ok(a.place, PLACE_MAX) {
        return Err(format!("an activity's place is 1–{PLACE_MAX} characters"));
    }
    if let Some(url) = a.meeting_url {
        if !a.online {
            return Err("only an online activity has a meeting link".into());
        }
        let web = url.starts_with("http://") || url.starts_with("https://");
        if !web || url.chars().any(char::is_whitespace) || !optional_text_ok(url, MEETING_URL_MAX) {
            return Err(format!(
                "a meeting link is an http(s) address of at most {MEETING_URL_MAX} characters"
            ));
        }
    }
    if !text_ok(a.note, LONG_TEXT_MAX) {
        return Err(format!(
            "an activity's note is 1–{LONG_TEXT_MAX} characters"
        ));
    }
    if a.roles.len() > ROLES_MAX {
        return Err(format!("an activity has at most {ROLES_MAX} roles"));
    }
    a.roles.iter().try_for_each(check_role)
}

fn check_role(r: &RoleSpec<'_>) -> Result<(), String> {
    if !text_ok(r.label, ROLE_LABEL_MAX) {
        return Err(format!("a role's name is 1–{ROLE_LABEL_MAX} characters"));
    }
    if !(1..=COUNT_MAX).contains(&r.required) {
        return Err(format!("a role needs 1–{COUNT_MAX} people"));
    }
    match r.maximum {
        Some(max) if max < r.required || max > COUNT_MAX => {
            Err("a role's maximum is at least the people it needs".into())
        }
        _ => Ok(()),
    }
}

/// Roles can be taken or left until the activity ends (web's `ensure_activity_roles_unlocked`).
#[must_use]
pub fn roles_open(ends_at: i64, at: i64) -> bool {
    at < ends_at
}

/// Ratings come after the activity ends (web's "Activity has not ended yet").
pub fn check_rating(stars: u8, comment: &str, ends_at: i64, at: i64) -> Result<(), String> {
    if at < ends_at {
        return Err("an activity can be rated once it has ended".into());
    }
    if !(1..=5).contains(&stars) {
        return Err("a rating is 1–5 stars".into());
    }
    if optional_text_ok(comment, COMMENT_MAX) {
        Ok(())
    } else {
        Err(format!(
            "a rating's comment is at most {COMMENT_MAX} characters"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> ActivitySpec<'static> {
        ActivitySpec {
            title: "Sunday repairs",
            starts_at: 10,
            ends_at: 20,
            online: false,
            place: "Shed",
            meeting_url: None,
            note: "Bring tools.",
            roles: vec![RoleSpec {
                label: "Mechanic",
                required: 2,
                maximum: Some(3),
            }],
        }
    }

    #[test]
    fn accepts_web_shaped_activities() {
        assert_eq!(check_activity(&spec()), Ok(()));
        let online = ActivitySpec {
            online: true,
            meeting_url: Some("https://meet.example/x"),
            ..spec()
        };
        assert_eq!(check_activity(&online), Ok(()));
    }

    #[test]
    fn refuses_what_web_refuses() {
        assert!(check_activity(&ActivitySpec {
            ends_at: 10,
            ..spec()
        })
        .is_err());
        assert!(check_activity(&ActivitySpec {
            title: " ",
            ..spec()
        })
        .is_err());
        assert!(check_activity(&ActivitySpec {
            meeting_url: Some("https://x"),
            ..spec()
        })
        .is_err());
        let ftp = ActivitySpec {
            online: true,
            meeting_url: Some("ftp://x"),
            ..spec()
        };
        assert!(check_activity(&ftp).is_err());
        let low = RoleSpec {
            label: "Cook",
            required: 2,
            maximum: Some(1),
        };
        assert!(check_activity(&ActivitySpec {
            roles: vec![low],
            ..spec()
        })
        .is_err());
        let none = RoleSpec {
            label: "Cook",
            required: 0,
            maximum: None,
        };
        assert!(check_activity(&ActivitySpec {
            roles: vec![none],
            ..spec()
        })
        .is_err());
    }

    #[test]
    fn locks_by_time() {
        assert!(roles_open(20, 19) && !roles_open(20, 20));
        assert!(check_rating(5, "", 20, 19).is_err());
        assert!(check_rating(0, "", 20, 20).is_err());
        assert_eq!(check_rating(4, "Good day", 20, 25), Ok(()));
    }
}
