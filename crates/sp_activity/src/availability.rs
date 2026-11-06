//! A personal service's listed availability and bookings (web-backend
//! `projects_service_availability.py`, `update_service_request_status`; ADR 0077). Weekly hours
//! are rules the UI expands into slots in their own time zone; a one-off slot is an ordinary
//! activity; an accepted timed request becomes a booking: an activity whose one role is the
//! provider's.

use sp_core::limits::optional_text_ok;

/// web's weekly hours note (`maxlength="120"`).
pub const RULE_NOTE_MAX: usize = 120;
/// web-backend `AvailabilityRuleCreateIn.timezone`.
pub const ZONE_MAX: usize = 64;
/// Minutes in a day: a rule's times are minutes after midnight, `0..MINUTES_PER_DAY`.
pub const MINUTES_PER_DAY: u16 = 24 * 60;
/// The label of a booking's one role (web-backend's `"Provider"`).
pub const PROVIDER: &str = "Provider";

/// web-backend `AvailabilityRuleCreateIn` + `create_availability_rule`: a weekday 0–6
/// (Monday first), start and end minutes that differ (an end before the start runs past
/// midnight), an IANA zone name and an optional note.
pub fn check_rule(weekday: u8, start: u16, end: u16, zone: &str, note: &str) -> Result<(), String> {
    if weekday > 6 {
        return Err("weekday must be 0-6".into());
    }
    if start >= MINUTES_PER_DAY || end >= MINUTES_PER_DAY {
        return Err("a time is minutes after midnight".into());
    }
    if start == end {
        return Err("end_time must differ from start_time".into());
    }
    let zone_ok = zone
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "/_+-".contains(c));
    if zone.is_empty() || !zone_ok || !optional_text_ok(zone, ZONE_MAX) {
        return Err(format!(
            "a time zone is an IANA name of at most {ZONE_MAX} characters"
        ));
    }
    if optional_text_ok(note, RULE_NOTE_MAX) {
        Ok(())
    } else {
        Err(format!("a note is at most {RULE_NOTE_MAX} characters"))
    }
}

/// A booking's roles: exactly web's one `Provider` role for one person.
pub fn check_booking(roles: &[crate::RoleSpec<'_>]) -> Result<(), String> {
    match roles {
        [r] if r.label == PROVIDER && r.required == 1 && r.maximum == Some(1) => Ok(()),
        _ => Err("a booking has one Provider role for one person".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RoleSpec;

    #[test]
    fn rules_follow_web() {
        assert_eq!(
            check_rule(0, 9 * 60, 12 * 60, "Australia/Melbourne", ""),
            Ok(())
        );
        assert_eq!(check_rule(6, 22 * 60, 60, "UTC", "Late"), Ok(()));
        assert!(check_rule(7, 0, 60, "UTC", "").is_err());
        assert!(check_rule(1, 60, 60, "UTC", "").is_err());
        assert!(check_rule(1, 0, MINUTES_PER_DAY, "UTC", "").is_err());
        assert!(check_rule(1, 0, 60, "", "").is_err());
        assert!(check_rule(1, 0, 60, "Etc/GMT+10", "").is_ok());
        assert!(check_rule(1, 0, 60, "<b>", "").is_err());
        assert!(check_rule(1, 0, 60, "UTC", &"x".repeat(121)).is_err());
    }

    #[test]
    fn bookings_have_one_provider() {
        let provider = RoleSpec {
            label: PROVIDER,
            required: 1,
            maximum: Some(1),
            description: "",
        };
        assert_eq!(check_booking(std::slice::from_ref(&provider)), Ok(()));
        assert!(check_booking(&[]).is_err());
        assert!(check_booking(&[provider.clone(), provider.clone()]).is_err());
        let open = RoleSpec {
            maximum: None,
            ..provider
        };
        assert!(check_booking(&[open]).is_err());
    }
}
