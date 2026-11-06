//! Service requests (web-backend's `projects_service_requests.py` and
//! `projects_service_request_settings.py`, ADR 0076). A request and its status lines live in
//! the direct chat between the person asking and the service's creator, so only those two see
//! it; a service's request settings are a proposal in the project. Shared by the integrity
//! zomes (validation) and the coordinators (input checks, replay, chat lines).
//! Not in `sp_core`: adding code there changes every DNA's hash.

mod settings;

pub use settings::{admits, check_settings, Settings, MODES, SERVICE_MODES};

/// web-backend `ServiceRequestCreateIn.title` (`max_length=200`).
pub const TITLE_MAX: usize = 200;
/// The chat's message limit (web's body has none; the request also becomes a message).
pub const BODY_MAX: usize = 3_000;
/// web-backend `ServiceRequestStatusUpdateIn.reason` (`max_length=500`).
pub const REASON_MAX: usize = 500;
/// A project id (an action hash in base64, or a private seed).
pub const PROJECT_MAX: usize = 200;

/// The decisions a request can get after it is sent ("open", web's Pending).
pub const STATUSES: [&str; 3] = ["accepted", "declined", "cancelled"];

/// A request names its service (id and title), has web's title and body, and either no
/// time or a start before its finish (microseconds).
pub fn check_request(
    project: &str,
    service: &str,
    title: &str,
    body: &str,
    window: Option<(i64, i64)>,
) -> Result<(), String> {
    if project.trim().is_empty() || project.len() > PROJECT_MAX {
        return Err("A request names its service".into());
    }
    if service.trim().is_empty() || service.chars().count() > TITLE_MAX {
        return Err("A request names its service".into());
    }
    if title.trim().is_empty() || title.chars().count() > TITLE_MAX {
        return Err(format!(
            "A request needs a title of at most {TITLE_MAX} characters"
        ));
    }
    if body.trim().is_empty() || body.chars().count() > BODY_MAX {
        return Err(format!(
            "A request needs details of at most {BODY_MAX} characters"
        ));
    }
    if let Some((start, end)) = window {
        if end <= start {
            return Err("The finish time must be after the start time".into());
        }
    }
    Ok(())
}

/// A known decision; only a decline carries a reason (web stores it only then).
pub fn check_status(status: &str, reason: &str) -> Result<(), String> {
    if !STATUSES.contains(&status) {
        return Err("Unknown request status".into());
    }
    if reason.chars().count() > REASON_MAX {
        return Err(format!("A reason is at most {REASON_MAX} characters"));
    }
    if status != "declined" && !reason.is_empty() {
        return Err("Only a decline has a reason".into());
    }
    Ok(())
}

/// Who may write a decision: the person who asked cancels; the other person (the service's
/// creator) accepts or declines (web's `_ensure_request_reviewer`).
#[must_use]
pub fn may_decide(status: &str, by_requester: bool) -> bool {
    (status == "cancelled") == by_requester
}

/// Where a request stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// web's "open" (shown as Pending).
    Pending,
    Accepted,
    Declined {
        reason: String,
    },
    Cancelled,
}

impl State {
    /// web's status ids.
    #[must_use]
    pub fn id(&self) -> &'static str {
        match self {
            Self::Pending => "open",
            Self::Accepted => "accepted",
            Self::Declined { .. } => "declined",
            Self::Cancelled => "cancelled",
        }
    }
}

/// web's `SERVICE_REQUEST_STATUS_LABELS`.
#[must_use]
pub fn status_label(id: &str) -> &str {
    match id {
        "open" | "planned" => "Pending",
        "accepted" => "Accepted",
        "declined" => "Declined",
        "cancelled" => "Cancelled",
        other => other,
    }
}

/// Replay a request's decisions, oldest first, as `(status, reason, by_requester)`: the
/// first one its writer may make settles it (web offers Accept, Decline and Cancel only while
/// a request is pending); later ones change nothing.
#[must_use]
pub fn replay<'a>(decisions: impl IntoIterator<Item = (&'a str, &'a str, bool)>) -> State {
    for (status, reason, by_requester) in decisions {
        if check_status(status, reason).is_err() || !may_decide(status, by_requester) {
            continue;
        }
        return match status {
            "accepted" => State::Accepted,
            "declined" => State::Declined {
                reason: reason.to_string(),
            },
            _ => State::Cancelled,
        };
    }
    State::Pending
}

/// web's `_format_personal_service_request_message`: the chat line a new request posts.
/// `when` is the window already formatted (web prints ISO times).
#[must_use]
pub fn request_message(title: &str, body: &str, when: Option<(&str, &str)>, href: &str) -> String {
    let mut lines = vec![
        identity_line(title, "open"),
        String::new(),
        body.trim().to_string(),
    ];
    if let Some((start, end)) = when {
        lines.extend([String::new(), format!("When: {start} – {end}")]);
    }
    lines.extend([String::new(), format!("View request: {href}")]);
    lines.join("\n")
}

/// web's `_format_status_line`: the chat line a decision posts.
#[must_use]
pub fn status_message(title: &str, status: &str, reason: &str, href: &str) -> String {
    let mut lines = vec![identity_line(title, status)];
    if status == "declined" && !reason.trim().is_empty() {
        lines.extend([String::new(), format!("Reason: {}", reason.trim())]);
    }
    lines.extend([String::new(), format!("View request: {href}")]);
    lines.join("\n")
}

fn identity_line(title: &str, status: &str) -> String {
    format!("Request \"{}\" · {}", title.trim(), status_label(status))
}

#[cfg(test)]
mod tests;
