//! Software projects (web-backend's `services/projects/software`, ADR 0086): pull requests
//! members vote on, merges a merge-capable member records and members confirm, who may merge
//! (grant or revoke votes) and the official repository (replacement votes). Each is a
//! proposal in the project; this crate checks their fields and replays the decisions into
//! what web shows. Shared by the integrity zome (fields) and the governance coordinator
//! (replay). Not in `sp_core`: adding code there changes every DNA's hash.

mod replay;

pub use replay::{replay, Event, Merge, Merger, PullRequest, Software, Source};

use serde::{Deserialize, Serialize};

/// web-backend `PullRequestSubmitIn.title` (`max_length=200`).
pub const TITLE_MAX: usize = 200;
/// web-backend `PullRequestSubmitIn.pullRequestId` (`max_length=120`); a merge ID too (its
/// column is `VARCHAR(120)`).
pub const ID_MAX: usize = 120;
/// Links (web has no limit; the plan's repository field allows 500).
pub const URL_MAX: usize = 500;
/// A summary or reason (web has no limit; the app's long-text limit).
pub const TEXT_MAX: usize = 20_000;

/// The modes whose projects can become software projects: web's productive projects and
/// collective services (`ProductiveLifecyclePhaseFive`, `CollectiveServicePhaseFive`).
pub const MODES: [&str; 2] = ["productive", "collective-service"];

/// The subtype a production or operations plan picks for a software project.
pub const SUBTYPE: &str = "software";

/// A pull request's stage (web's `PR_STAGE_LABELS`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    /// Members vote on the pull request.
    Approval,
    /// Approved: a merge-capable member records the merge.
    AwaitingMerge,
    /// A merge is recorded: members confirm it.
    Confirmation,
    Confirmed,
    Rejected,
    /// A repository replacement it triggered passed.
    Replaced,
}

impl Stage {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Approval => "Vote needed",
            Self::AwaitingMerge => "Merge needed",
            Self::Confirmation => "Merge confirmation needed",
            Self::Confirmed => "Merged",
            Self::Rejected => "Rejected",
            Self::Replaced => "Replaced",
        }
    }

    /// web's in-progress pull requests (`isInProgressPullRequest`).
    #[must_use]
    pub fn in_progress(self) -> bool {
        matches!(
            self,
            Self::Approval | Self::AwaitingMerge | Self::Confirmation
        )
    }
}

fn text(value: &str, max: usize, what: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.chars().count() > max {
        return Err(format!("{what} is required and at most {max} characters"));
    }
    Ok(())
}

/// web-backend's `PullRequestSubmitIn`: every field is required.
pub fn check_pull_request(title: &str, summary: &str, id: &str, url: &str) -> Result<(), String> {
    text(title, TITLE_MAX, "A pull request's title")?;
    text(summary, TEXT_MAX, "A pull request's summary")?;
    text(id, ID_MAX, "A pull request's ID")?;
    text(url, URL_MAX, "A pull request's link")
}

/// web-backend's `MergeRecordIn`: a merge ID and a link others can open.
pub fn check_merge(id: &str, url: &str) -> Result<(), String> {
    text(id, ID_MAX, "A merge ID")?;
    text(url, URL_MAX, "A merge link")
}

/// web-backend's `RepositoryReplacementRequestIn`: the new repository and why.
pub fn check_repository(url: &str, reason: &str) -> Result<(), String> {
    text(url, URL_MAX, "The replacement repository")?;
    text(reason, TEXT_MAX, "A reason")
}

/// Whether a project of `mode` takes software proposals.
#[must_use]
pub fn takes_software(mode: &str) -> bool {
    MODES.contains(&mode)
}

#[cfg(test)]
mod tests;
