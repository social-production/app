//! Who may create projects (docs/plans/M2-projects.md): anyone; New members at most
//! `NEW_PROJECT_CAP` per rolling `NEW_PROJECT_WINDOW_US`; Full members without limit.

const DAY_US: i64 = 24 * 60 * 60 * 1_000_000;

pub const NEW_PROJECT_CAP: usize = 2;
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
    fn two_per_week_for_new_members() {
        let now = 100 * DAY_US;
        assert!(check_new_member_project_cap(&[], now).is_ok());
        assert!(check_new_member_project_cap(&[now - DAY_US], now).is_ok());
        assert!(check_new_member_project_cap(&[now - DAY_US, now - 2 * DAY_US], now).is_err());
        // Older than 7 days no longer counts.
        assert!(check_new_member_project_cap(&[now - 8 * DAY_US, now - 2 * DAY_US], now).is_ok());
    }
}
