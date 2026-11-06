//! A service's request settings (web's `project_service_request_settings`): whether it takes
//! requests, how (listed slots, written messages or both), and whether a timed request may
//! fall outside the listed slots.

/// web-backend `VALID_REQUEST_MODES`.
pub const MODES: [&str; 3] = ["calendar", "direct", "both"];
/// The project modes that take service requests (web's `_ensure_service_project`).
pub const SERVICE_MODES: [&str; 2] = ["collective-service", "personal-service"];
/// A settings vote's reason (the app's long-text limit).
pub const REASON_MAX: usize = 4_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub enabled: bool,
    pub mode: String,
    pub off_schedule: bool,
}

impl Settings {
    /// web's defaults when a project has no settings row: a personal service takes requests
    /// of both kinds, off-schedule too; a collective service takes none until a vote passes.
    #[must_use]
    pub fn default_for(project_mode: &str) -> Self {
        let personal = project_mode == "personal-service";
        Self {
            enabled: personal,
            mode: "both".into(),
            off_schedule: personal,
        }
    }

    /// web's create flow: a new personal service starts with the chosen mode, off-schedule
    /// requests on only for "both" (`helpers.py`).
    #[must_use]
    pub fn created_with(mode: &str) -> Self {
        Self {
            enabled: true,
            mode: mode.into(),
            off_schedule: mode == "both",
        }
    }
}

/// A known mode; a governed change (a collective service's vote) needs a reason, a personal
/// service's creator saves without one.
pub fn check_settings(project_mode: &str, mode: &str, reason: &str) -> Result<(), String> {
    if !SERVICE_MODES.contains(&project_mode) {
        return Err("Only service projects support service requests".into());
    }
    if !MODES.contains(&mode) {
        return Err("Unknown request mode".into());
    }
    if reason.chars().count() > REASON_MAX {
        return Err(format!("A reason is at most {REASON_MAX} characters"));
    }
    if project_mode != "personal-service" && reason.trim().is_empty() {
        return Err("A reason is required for governed request settings changes".into());
    }
    Ok(())
}

/// web's `_ensure_requests_enabled`, without the listed-slot check (availability comes later,
/// ADR 0076 part b): whether these settings take a request with or without a time.
pub fn admits(settings: &Settings, timed: bool) -> Result<(), String> {
    if !settings.enabled {
        return Err("Not taking requests.".into());
    }
    if settings.mode == "calendar" && !timed {
        return Err("Calendar requests require a start and finish time".into());
    }
    if settings.mode == "direct" && timed && !settings.off_schedule {
        return Err("This service only accepts direct unscheduled requests".into());
    }
    Ok(())
}
