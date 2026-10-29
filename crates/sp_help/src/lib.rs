//! Help requests (ADR 0067): commons spaces whose charter mode is `"help-request"`, carrying one
//! activity with the request's time, place and roles. Ported from web-backend
//! `services/content/help_requests.py`.

/// The charter mode and subtype of every help request.
pub const MODE: &str = "help-request";
pub const SUBTYPE: &str = "standard";
pub fn check_kind(mode: &str, subtype: &str) -> Result<(), String> {
    if mode == MODE && subtype == SUBTYPE {
        Ok(())
    } else {
        Err(format!(
            "a help request has mode {MODE:?} and subtype {SUBTYPE:?}"
        ))
    }
}

/// web's slots as an activity role's (required, maximum): 0 means any number of people.
#[must_use]
pub fn role_counts(slots: u32) -> (u32, Option<u32>) {
    if slots == 0 {
        (1, None)
    } else {
        (slots, Some(slots))
    }
}

/// An activity role's slots, as web shows them (0 = any number).
#[must_use]
pub fn slots(maximum: Option<u32>) -> u32 {
    maximum.unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind() {
        assert!(check_kind(MODE, SUBTYPE).is_ok());
        assert!(check_kind("event", SUBTYPE).is_err());
        assert!(check_kind(MODE, "software").is_err());
    }

    #[test]
    fn slots_round_trip() {
        assert_eq!(role_counts(0), (1, None));
        assert_eq!(role_counts(3), (3, Some(3)));
        assert_eq!(slots(role_counts(0).1), 0);
        assert_eq!(slots(role_counts(4).1), 4);
    }
}
