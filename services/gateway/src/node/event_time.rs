//! An event card's time (web's `EventCard` `scheduledAt`): web-backend copies the leading
//! plan's start onto the event when that plan leads (`_sync_event_schedule_from_leading_plan`);
//! the index reads it from the event's plan board instead.

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
    pub schedule: Option<Schedule>,
}

/// Mirrors `project_integrity::EventSchedule` (UTC dates and times, ADR 0057).
#[derive(Debug, Deserialize)]
pub struct Schedule {
    pub start_date: Option<String>,
    pub start_time: Option<String>,
}

/// The leading approved event plan's start, in Holochain microseconds.
pub fn event_start(board: &PlanBoard) -> Option<i64> {
    let lead = board
        .plans
        .iter()
        .find(|p| p.plan.kind == "Event" && p.approved && p.leader.as_deref() == Some("Leading"))?;
    let s = lead.plan.schedule.as_ref()?;
    utc_micros(
        s.start_date.as_deref()?,
        s.start_time.as_deref().unwrap_or("00:00"),
    )
}

/// `YYYY-MM-DD` and `HH:MM` in UTC as microseconds since the epoch.
fn utc_micros(date: &str, time: &str) -> Option<i64> {
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
                r#"{{"plan":{{"kind":"Event","schedule":{{"start_date":"{date}","start_time":"09:30"}}}},"approved":{approved},"leader":{leader}}}"#
            )
        };
        let b = board(&format!(
            r#"{{"plans":[{},{},{}]}}"#,
            plan(false, r#""Leading""#, "2026-11-01"),
            plan(true, "null", "2026-11-02"),
            plan(true, r#""Leading""#, "2026-11-14"),
        ));
        assert_eq!(event_start(&b), Some(1_794_648_600_000_000));
        assert_eq!(event_start(&board(r#"{"plans":[]}"#)), None);
    }
}
