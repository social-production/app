//! Events (ADR 0057): project clones whose charter mode is `"event"`. The lifecycle lives in
//! `sp_governance` (mode `EVENT`); this crate holds the fields only events have, ported from
//! web-backend `routers/events_plans.py` and `utils/plan_schedule.py`.

use sp_core::plan::is_date;

/// The charter mode and subtype of every event.
pub const MODE: &str = "event";
pub const SUBTYPE: &str = "standard";

/// web's `EventPlanScheduleMode`.
pub const SCHEDULE_MODES: [&str; 3] = ["any-day", "date", "range"];

/// An event plan's schedule, in UTC wall time (web's `schedule_to_utc`): the device converts
/// from and to the display time zone.
#[derive(Clone, Copy, Debug)]
pub struct ScheduleFields<'a> {
    pub mode: &'a str,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
    pub start_time: Option<&'a str>,
    pub finish_time: Option<&'a str>,
}

/// `Err` names the first field that breaks the rules (web's plan wizard steps for events).
pub fn check_schedule(s: &ScheduleFields<'_>) -> Result<(), String> {
    if !SCHEDULE_MODES.contains(&s.mode) {
        return Err(format!("unknown schedule mode {:?}", s.mode));
    }
    if [s.start_date, s.end_date]
        .into_iter()
        .flatten()
        .any(|d| !is_date(d))
    {
        return Err("schedule dates must be YYYY-MM-DD".into());
    }
    if [s.start_time, s.finish_time]
        .into_iter()
        .flatten()
        .any(|t| !is_time(t))
    {
        return Err("schedule times must be HH:MM".into());
    }
    match (s.mode, s.start_date, s.end_date) {
        ("any-day", None, None) | ("date", Some(_), _) => Ok(()),
        ("range", Some(start), Some(end)) if end >= start => Ok(()),
        ("any-day", _, _) => Err("an any-day schedule has no dates".into()),
        ("date", None, _) => Err("a dated schedule needs its date".into()),
        _ => Err("a range needs a start date and an end date on or after it".into()),
    }
}

/// `HH:MM`, 00:00–23:59 (the shape a time input sends).
#[must_use]
pub fn is_time(text: &str) -> bool {
    let Some((h, m)) = text.split_once(':') else {
        return false;
    };
    let two = |s: &str| s.len() == 2 && s.bytes().all(|b| b.is_ascii_digit());
    two(h) && two(m) && h < "24" && m < "60"
}

/// Who an event is for and who runs it, read from its network seed (ADR 0061).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Audience {
    /// `<creator>.<nonce>`: listed in the directory, collaborative.
    Public,
    /// `<creator>.private.<nonce>`: invite only, collaborative.
    Private,
    /// `<creator>.organized.<nonce>`: invite only, the creator and co-organizers decide.
    Organized,
}

/// The seed word after the creator for each private audience.
pub const PRIVATE: &str = "private";
pub const ORGANIZED: &str = "organized";

impl Audience {
    /// `<creator>.<word>.<nonce>` with a private word, else public. A project seed's nonce
    /// never contains a dot, so older seeds stay public.
    #[must_use]
    pub fn of_seed(seed: &str) -> Self {
        let mut parts = seed.splitn(3, '.');
        let (_creator, word, nonce) = (parts.next(), parts.next(), parts.next());
        match (word, nonce) {
            (Some(PRIVATE), Some(n)) if !n.is_empty() => Self::Private,
            (Some(ORGANIZED), Some(n)) if !n.is_empty() => Self::Organized,
            _ => Self::Public,
        }
    }

    #[must_use]
    pub fn is_private(self) -> bool {
        self != Self::Public
    }

    #[must_use]
    pub fn organized(self) -> bool {
        self == Self::Organized
    }

    /// web's `audience` (`private_community` waits for communities, M5d).
    #[must_use]
    pub fn audience_id(self) -> &'static str {
        if self.is_private() {
            "invite_only"
        } else {
            "public"
        }
    }

    /// web's `governance`.
    #[must_use]
    pub fn governance_id(self) -> &'static str {
        if self.organized() {
            "organizer_controlled"
        } else {
            "collaborative"
        }
    }
}

/// An event's charter and listing use mode `"event"` with subtype `"standard"`.
pub fn check_kind(mode: &str, subtype: &str) -> Result<(), String> {
    if mode == MODE && subtype == SUBTYPE {
        Ok(())
    } else {
        Err(format!(
            "an event has mode {MODE:?} and subtype {SUBTYPE:?}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schedule<'a>(
        mode: &'a str,
        start: Option<&'a str>,
        end: Option<&'a str>,
    ) -> ScheduleFields<'a> {
        ScheduleFields {
            mode,
            start_date: start,
            end_date: end,
            start_time: Some("09:30"),
            finish_time: Some("17:00"),
        }
    }

    #[test]
    fn schedules_follow_the_wizard() {
        assert!(check_schedule(&schedule("any-day", None, None)).is_ok());
        assert!(check_schedule(&schedule("date", Some("2026-11-02"), None)).is_ok());
        assert!(check_schedule(&schedule("range", Some("2026-11-02"), Some("2026-11-04"))).is_ok());
        assert!(
            check_schedule(&schedule("range", Some("2026-11-04"), Some("2026-11-02"))).is_err()
        );
        assert!(check_schedule(&schedule("date", None, None)).is_err());
        assert!(check_schedule(&schedule("any-day", Some("2026-11-02"), None)).is_err());
        assert!(check_schedule(&schedule("weekly", None, None)).is_err());
        assert!(check_schedule(&schedule("date", Some("2026-13-02"), None)).is_err());
    }

    #[test]
    fn times_are_hours_and_minutes() {
        assert!(is_time("00:00") && is_time("23:59"));
        assert!(!is_time("24:00") && !is_time("9:30") && !is_time("12:60") && !is_time("ab:cd"));
        let mut s = schedule("any-day", None, None);
        s.finish_time = Some("5pm");
        assert!(check_schedule(&s).is_err());
    }

    #[test]
    fn the_seed_names_the_audience() {
        assert_eq!(Audience::of_seed("uhCAkme.abc123"), Audience::Public);
        assert_eq!(Audience::of_seed("uhCAkme.private.abc"), Audience::Private);
        assert_eq!(
            Audience::of_seed("uhCAkme.organized.abc"),
            Audience::Organized
        );
        assert_eq!(Audience::of_seed("uhCAkme.organized."), Audience::Public);
        assert_eq!(Audience::of_seed("uhCAkme.direct.abc"), Audience::Public);
        assert!(Audience::Organized.is_private() && Audience::Organized.organized());
        assert!(Audience::Private.is_private() && !Audience::Private.organized());
    }

    #[test]
    fn events_are_standard() {
        assert!(check_kind("event", "standard").is_ok());
        assert!(check_kind("event", "software").is_err());
        assert!(check_kind("productive", "standard").is_err());
    }
}
