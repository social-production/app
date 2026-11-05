//! Content reports (ADR 0071), ported from web-backend's `services/moderation`: a report names
//! a reason (spam or serious harm) and what's wrong; people vote yes (take it down) or no. The
//! report's state follows web's `advance_resolution`, replayed over the votes in time order
//! (`replay`), with web's quorums and yes-shares (`thresholds`). Pure: no HDK, no I/O.

mod replay;
mod thresholds;

pub use replay::{age_days, replay, replay_chat, Ballot, Outcome};
pub use thresholds::{
    age_boost_percent, audience, delete_quorum, delete_share_percent, hide_quorum,
    hide_share_percent, popularity_boost_percent, popularity_score, Thresholds, MIN_AUDIENCE,
};

/// web's `ReportReason`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Spam,
    SeriousHarm,
}

impl Reason {
    #[must_use]
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "spam" => Some(Self::Spam),
            "serious-harm" => Some(Self::SeriousHarm),
            _ => None,
        }
    }

    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Spam => "spam",
            Self::SeriousHarm => "serious-harm",
        }
    }
}

/// web's `ReportResolution`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolution {
    Open,
    UnderReview,
    Hidden,
    Removed,
    Dismissed,
}

impl Resolution {
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::UnderReview => "under_review",
            Self::Hidden => "hidden",
            Self::Removed => "removed",
            Self::Dismissed => "dismissed",
        }
    }

    /// web's `ACTIVE_REPORT_RESOLUTIONS`: a later report waits until this one ends.
    #[must_use]
    pub fn active(self) -> bool {
        matches!(self, Self::Open | Self::UnderReview | Self::Hidden)
    }
}

/// The `ReportVotes` link tag: one byte.
pub const YES: u8 = 1;
pub const NO: u8 = 0;

#[must_use]
pub fn is_vote_tag(tag: &[u8]) -> bool {
    matches!(tag, [YES | NO])
}

/// The longest "What's wrong?" text, in characters (web's textarea has none; the DHT needs one).
pub const DESCRIPTION_MAX: usize = 2000;

/// web's composer: a known reason and a non-empty description.
///
/// # Errors
/// The reason is unknown, or the description is empty or too long.
pub fn check_report(reason: &str, description: &str) -> Result<(), String> {
    if Reason::parse(reason).is_none() {
        return Err("a report's reason is spam or serious harm".into());
    }
    let text = description.trim();
    if text.is_empty() {
        return Err("say what's wrong".into());
    }
    if text.chars().count() > DESCRIPTION_MAX {
        return Err(format!("keep it under {DESCRIPTION_MAX} characters"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_need_a_reason_and_text() {
        assert!(check_report("spam", "ads").is_ok());
        assert!(check_report("serious-harm", "threats").is_ok());
        assert!(check_report("rude", "x").is_err());
        assert!(check_report("spam", "   ").is_err());
        assert!(check_report("spam", &"x".repeat(DESCRIPTION_MAX + 1)).is_err());
    }

    #[test]
    fn vote_tags_are_one_byte() {
        assert!(is_vote_tag(&[YES]));
        assert!(is_vote_tag(&[NO]));
        assert!(!is_vote_tag(&[]));
        assert!(!is_vote_tag(&[2]));
        assert!(!is_vote_tag(&[1, 1]));
    }
}
