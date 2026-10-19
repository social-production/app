//! Calendar windows in the viewer's time zone, like web's `calendar_window_start/end`.
//! The viewer sends its UTC offset (minutes east of UTC) instead of a zone name.

use crate::{Micros, MICROS_PER_DAY};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Window {
    Today,
    Week,
    Month,
    All,
}

impl Window {
    /// web's `normalize_window`: unknown values mean all.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "today" => Self::Today,
            "week" => Self::Week,
            "month" => Self::Month,
            _ => Self::All,
        }
    }
}

/// `[start, end)` in UTC micros, or `None` for all time. Weeks start on Monday.
#[must_use]
pub fn window_bounds(
    window: Window,
    now: Micros,
    utc_offset_minutes: i32,
) -> Option<(Micros, Micros)> {
    let offset = i64::from(utc_offset_minutes) * 60_000_000;
    let local_day = (now + offset).div_euclid(MICROS_PER_DAY);
    let (start_day, end_day) = match window {
        Window::All => return None,
        Window::Today => (local_day, local_day + 1),
        Window::Week => {
            // 1970-01-01 was a Thursday: Monday-based weekday = (day + 3) mod 7.
            let monday = local_day - (local_day + 3).rem_euclid(7);
            (monday, monday + 7)
        }
        Window::Month => {
            let (y, m, _) = civil_from_days(local_day);
            let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
            (days_from_civil(y, m, 1), days_from_civil(ny, nm, 1))
        }
    };
    Some((
        start_day * MICROS_PER_DAY - offset,
        end_day * MICROS_PER_DAY - offset,
    ))
}

/// Howard Hinnant's `days_from_civil`: days since 1970-01-01 for a proleptic Gregorian date.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The inverse of [`days_from_civil`].
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (
        if month <= 2 {
            yoe + era * 400 + 1
        } else {
            yoe + era * 400
        },
        month,
        day,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: Micros = MICROS_PER_DAY;
    // 2026-10-18 (a Sunday) 13:00 UTC.
    const NOW: Micros = (20_744 * 24 + 13) * 3_600_000_000;

    #[test]
    fn civil_round_trip() {
        assert_eq!(days_from_civil(2026, 10, 18), 20_744);
        assert_eq!(civil_from_days(20_744), (2026, 10, 18));
        assert_eq!(civil_from_days(days_from_civil(2024, 2, 29)), (2024, 2, 29));
    }

    #[test]
    fn today_in_brisbane_starts_at_local_midnight() {
        // UTC+10: 13:00 UTC is 23:00 local on the 18th, so today began 2026-10-17 14:00 UTC.
        let (start, end) = window_bounds(Window::Today, NOW, 600).expect("bounded window");
        assert_eq!(start, 20_744 * DAY - 10 * 3_600_000_000);
        assert_eq!(end - start, DAY);
    }

    #[test]
    fn week_starts_on_monday() {
        let (start, end) = window_bounds(Window::Week, NOW, 0).expect("bounded window");
        assert_eq!(civil_from_days(start / DAY), (2026, 10, 12));
        assert_eq!(end - start, 7 * DAY);
    }

    #[test]
    fn month_spans_the_calendar_month() {
        let (start, end) = window_bounds(Window::Month, NOW, 0).expect("bounded window");
        assert_eq!(civil_from_days(start / DAY), (2026, 10, 1));
        assert_eq!(civil_from_days(end / DAY), (2026, 11, 1));
        let dec = (days_from_civil(2026, 12, 5)) * DAY;
        let (_, end) = window_bounds(Window::Month, dec, 0).expect("bounded window");
        assert_eq!(civil_from_days(end / DAY), (2027, 1, 1));
    }

    #[test]
    fn all_has_no_bounds() {
        assert_eq!(window_bounds(Window::parse("forever"), NOW, 0), None);
    }
}
