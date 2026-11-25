//! Who may create projects (docs/plans/M2-projects.md): anyone; New members at most
//! `NEW_PROJECT_CAP` per rolling `NEW_PROJECT_WINDOW_US`; Full members without limit.

const DAY_US: i64 = 24 * 60 * 60 * 1_000_000;

/// Ten a week until two vouches (owner, 2026-11-24; ADR 0120): room to start a real
/// group's worth of proposals, threads and help requests, still a brake on a bot's flood.
pub const NEW_PROJECT_CAP: usize = 10;
pub const NEW_PROJECT_WINDOW_US: i64 = 7 * DAY_US;

/// `earlier` holds the times of the creator's earlier listings; `now` is the new one's.
pub fn check_new_member_project_cap(earlier: &[i64], now: i64) -> Result<(), String> {
    let recent = earlier
        .iter()
        .filter(|t| **t > now - NEW_PROJECT_WINDOW_US && **t <= now)
        .count();
    if recent >= NEW_PROJECT_CAP {
        return Err(format!(
            "new members can start {NEW_PROJECT_CAP} projects per 7 days; full members have no limit"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_per_week_for_new_members() {
        let now = 100 * DAY_US;
        let nine: Vec<i64> = (1..10).map(|h| now - h * 3_600_000_000).collect();
        let ten: Vec<i64> = (1..11).map(|h| now - h * 3_600_000_000).collect();
        assert!(check_new_member_project_cap(&[], now).is_ok());
        assert!(check_new_member_project_cap(&nine, now).is_ok());
        assert!(check_new_member_project_cap(&ten, now).is_err());
        // Older than 7 days no longer counts.
        let mut aged = nine.clone();
        aged.push(now - 8 * DAY_US);
        assert!(check_new_member_project_cap(&aged, now).is_ok());
    }
}
