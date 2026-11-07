//! An event card's time and place (web's `EventCard` `scheduledAt` and `locationLabel`):
//! web-backend copies the leading plan's start and place onto the event when that plan leads
//! (`_sync_event_schedule_from_leading_plan`); the index reads them from the plan board.

use serde::Deserialize;

/// Mirrors the governance zome's `PlanBoard` (the fields the card needs).
#[derive(Debug, Deserialize)]
pub struct PlanBoard {
    pub plans: Vec<PlanView>,
}

#[derive(Debug, Deserialize)]
pub struct PlanView {
    pub plan: Plan,
    pub approved: bool,
    pub leader: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Plan {
    pub kind: String,
    pub location: Option<String>,
    /// ADR 0078; older plans have none.
    #[serde(default)]
    pub place: Option<super::markers::PlaceMirror>,
    pub schedule: Option<Schedule>,
}

/// Mirrors `project_integrity::EventSchedule` (UTC dates and times, ADR 0057).
#[derive(Debug, Deserialize)]
pub struct Schedule {
    pub start_date: Option<String>,
    pub start_time: Option<String>,
}

/// What the leading plan sets on an event's card.
#[derive(Debug, Default, PartialEq)]
pub struct EventFacts {
    /// The plan's start, in Holochain microseconds.
    pub start: Option<i64>,
    /// The plan's place, when it names one.
    pub place: Option<String>,
    /// Its coordinates, when the place was picked (ADR 0082).
    pub coordinates: Option<super::markers::CardPlace>,
    /// The plan's picked place is online (ADR 0084).
    pub online: bool,
}

impl EventFacts {
    /// web-backend's sync: the plan's start, and its place over the charter's.
    pub fn apply(self, card: &mut super::ProjectCard) {
        card.scheduled_at = self.start;
        if let Some(place) = self.place {
            // A new label without coordinates must not keep the charter's pin (web uses the
            // plan's location row alone).
            card.place = self.coordinates;
            card.online = self.online;
            card.location_label = place;
        }
    }
}

/// The leading approved event plan's start and place (nothing without one).
pub fn event_facts(board: &PlanBoard) -> EventFacts {
    let lead = board
        .plans
        .iter()
        .find(|p| p.plan.kind == "Event" && p.approved && p.leader.as_deref() == Some("Leading"));
    let Some(plan) = lead.map(|p| &p.plan) else {
        return EventFacts::default();
    };
    let start = plan.schedule.as_ref().and_then(|s| {
        utc_micros(
            s.start_date.as_deref()?,
            s.start_time.as_deref().unwrap_or("00:00"),
        )
    });
    let place = plan
        .location
        .as_deref()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned);
    let coordinates = plan
        .place
        .as_ref()
        .and_then(super::markers::PlaceMirror::physical);
    EventFacts {
        start,
        place,
        coordinates,
        online: plan.place.as_ref().is_some_and(|p| p.online),
    }
}

/// `YYYY-MM-DD` and `HH:MM` in UTC as microseconds since the epoch.
pub fn utc_micros(date: &str, time: &str) -> Option<i64> {
    let mut d = date.splitn(3, '-').map(str::parse::<i64>);
    let (y, m, day) = (d.next()?.ok()?, d.next()?.ok()?, d.next()?.ok()?);
    let (h, min) = time.split_once(':')?;
    let (h, min) = (h.parse::<i64>().ok()?, min.parse::<i64>().ok()?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&day) || h > 23 || min > 59 {
        return None;
    }
    let seconds = (days_from_civil(y, m, day) * 24 + h) * 3600 + min * 60;
    Some(seconds * 1_000_000)
}

/// Days since 1970-01-01 (Howard Hinnant's `days_from_civil`).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(json: &str) -> PlanBoard {
        serde_json::from_str(json).expect("a plan board")
    }

    #[test]
    fn dates_are_utc_micros() {
        assert_eq!(utc_micros("1970-01-01", "00:00"), Some(0));
        // 2026-11-14T09:30:00Z
        assert_eq!(
            utc_micros("2026-11-14", "09:30"),
            Some(1_794_648_600_000_000)
        );
        assert_eq!(utc_micros("2026-13-01", "09:00"), None);
        assert_eq!(utc_micros("soon", "09:00"), None);
    }

    #[test]
    fn the_leading_approved_event_plan_sets_the_time() {
        let plan = |approved: bool, leader: &str, date: &str| {
            format!(
                r#"{{"plan":{{"kind":"Event","location":"Hall {date}","schedule":{{"start_date":"{date}","start_time":"09:30"}}}},"approved":{approved},"leader":{leader}}}"#
            )
        };
        let b = board(&format!(
            r#"{{"plans":[{},{},{}]}}"#,
            plan(false, r#""Leading""#, "2026-11-01"),
            plan(true, "null", "2026-11-02"),
            plan(true, r#""Leading""#, "2026-11-14"),
        ));
        let facts = event_facts(&b);
        assert_eq!(facts.start, Some(1_794_648_600_000_000));
        assert_eq!(facts.place.as_deref(), Some("Hall 2026-11-14"));
        assert_eq!(
            event_facts(&board(r#"{"plans":[]}"#)),
            EventFacts::default()
        );
    }
}
