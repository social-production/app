//! Plan fields, as web's plan wizard collects them (web-backend `submit_project_plan`).
//! Shared limits for the project DNA's `Plan` proposals.

use crate::limits::{optional_text_ok, text_ok, LONG_TEXT_MAX, SHORT_TEXT_MAX};
use crate::project::{LOCATION_LABEL_MAX, SUBTYPES};

pub const PLAN_TITLE_MAX: usize = SHORT_TEXT_MAX;
pub const PLAN_TEXT_MAX: usize = LONG_TEXT_MAX;
pub const REPOSITORY_URL_MAX: usize = 500;
pub const RECIPIENTS_MAX: usize = 500;
pub const STAGES_MAX: usize = 30;

/// One stage of a plan; dates are web's `YYYY-MM-DD` date inputs (optional).
#[derive(Clone, Copy, Debug)]
pub struct StageFields<'a> {
    pub title: &'a str,
    pub details: &'a str,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub struct PlanFields<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub demand_note: &'a str,
    pub subtype: Option<&'a str>,
    pub repository_url: Option<&'a str>,
    pub location: Option<&'a str>,
    pub recipients: Option<&'a str>,
    pub stages: Vec<StageFields<'a>>,
}

/// `Err` names the first field that breaks the rules (web's wizard requires the same fields).
pub fn check_plan(f: &PlanFields<'_>) -> Result<(), String> {
    if !text_ok(f.title, PLAN_TITLE_MAX) {
        return Err(format!(
            "a plan title must be 1–{PLAN_TITLE_MAX} characters"
        ));
    }
    if !text_ok(f.description, PLAN_TEXT_MAX) || !text_ok(f.demand_note, PLAN_TEXT_MAX) {
        return Err("a plan needs a description and a note on current support".into());
    }
    if f.subtype.is_some_and(|s| !SUBTYPES.contains(&s)) {
        return Err("unknown plan subtype".into());
    }
    let fits = |text: Option<&str>, max: usize| text.is_none_or(|t| optional_text_ok(t, max));
    if !fits(f.repository_url, REPOSITORY_URL_MAX)
        || !fits(f.location, LOCATION_LABEL_MAX)
        || !fits(f.recipients, RECIPIENTS_MAX)
    {
        return Err("a plan's repository, location or recipients are too long".into());
    }
    if f.stages.is_empty() || f.stages.len() > STAGES_MAX {
        return Err(format!("a plan has 1–{STAGES_MAX} stages"));
    }
    for stage in &f.stages {
        let dates_ok = [stage.start_date, stage.end_date]
            .into_iter()
            .flatten()
            .all(is_date);
        if !text_ok(stage.title, PLAN_TITLE_MAX)
            || !text_ok(stage.details, PLAN_TEXT_MAX)
            || !dates_ok
        {
            return Err("each stage needs a title and details, and dates as YYYY-MM-DD".into());
        }
    }
    Ok(())
}

/// `YYYY-MM-DD` with plausible month and day (the shape a date input sends).
#[must_use]
pub fn is_date(text: &str) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    let [y, m, d] = parts.as_slice() else {
        return false;
    };
    let digits = |s: &str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
    if !(digits(y, 4) && digits(m, 2) && digits(d, 2)) {
        return false;
    }
    let month: u8 = m.parse().unwrap_or(0);
    let day: u8 = d.parse().unwrap_or(0);
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage<'a>(title: &'a str, start: Option<&'a str>) -> StageFields<'a> {
        StageFields {
            title,
            details: "Dig the beds.",
            start_date: start,
            end_date: None,
        }
    }

    fn plan(stages: Vec<StageFields<'_>>) -> PlanFields<'_> {
        PlanFields {
            title: "Raised beds",
            description: "Build six raised beds.",
            demand_note: "Meets the support we have.",
            subtype: Some("standard"),
            repository_url: None,
            location: Some("Allotment 4"),
            recipients: None,
            stages,
        }
    }

    #[test]
    fn accepts_a_wizard_plan() {
        assert_eq!(
            check_plan(&plan(vec![stage("Build", Some("2026-11-01"))])),
            Ok(())
        );
    }

    #[test]
    fn rejects_missing_or_bad_fields() {
        assert!(check_plan(&plan(vec![])).is_err(), "needs a stage");
        assert!(check_plan(&plan(vec![stage(" ", None)])).is_err());
        assert!(check_plan(&plan(vec![stage("Build", Some("1 Nov"))])).is_err());
        let mut bad = plan(vec![stage("Build", None)]);
        bad.subtype = Some("commercial");
        assert!(check_plan(&bad).is_err());
        let mut blank = plan(vec![stage("Build", None)]);
        blank.demand_note = "  ";
        assert!(check_plan(&blank).is_err());
    }

    #[test]
    fn dates_are_iso_days() {
        assert!(is_date("2026-02-28"));
        assert!(!is_date("2026-13-01") && !is_date("26-01-01") && !is_date("2026-01-1"));
    }
}
