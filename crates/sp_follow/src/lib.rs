//! Follows between people (web-backend `services/users.py`, ADR 0054). A follow is a link the
//! follower writes; the person followed may require approval and then accepts or rejects it
//! with links of their own. Times are microseconds; each decision counts only when it is
//! newer than the follow, so following again after a rejection asks again (as on web, where
//! unfollowing deletes the row).

/// Where one follow stands, from the follower's point of view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Counted: web's `accepted`.
    Accepted,
    /// Waiting for the person followed (web's `pending`).
    Pending,
    /// Declined; shown as not following (web deletes the row).
    Rejected,
}

/// What is known about one follow.
#[derive(Clone, Copy, Debug, Default)]
pub struct Facts {
    /// When the follower made the follow.
    pub followed_at: i64,
    /// Since when the person followed requires approval (`None`: they don't).
    pub approval_since: Option<i64>,
    /// Their newest accept and reject for this follower, if any.
    pub accepted_at: Option<i64>,
    pub rejected_at: Option<i64>,
}

/// The newest decision made after the follow wins; without one, a follow made while approval
/// was required waits, and any other follow counts at once (web stores the status at follow
/// time, so turning approval on later doesn't unfollow anyone).
#[must_use]
pub fn status(f: &Facts) -> Status {
    let after = |t: Option<i64>| t.filter(|&t| t >= f.followed_at);
    match (after(f.accepted_at), after(f.rejected_at)) {
        (Some(a), Some(r)) if r > a => Status::Rejected,
        (Some(_), _) => Status::Accepted,
        (None, Some(_)) => Status::Rejected,
        (None, None) => match f.approval_since {
            Some(since) if since <= f.followed_at => Status::Pending,
            _ => Status::Accepted,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{status, Facts, Status};

    fn facts(approval_since: Option<i64>) -> Facts {
        Facts {
            followed_at: 100,
            approval_since,
            ..Facts::default()
        }
    }

    #[test]
    fn open_profiles_accept_at_once() {
        assert_eq!(status(&facts(None)), Status::Accepted);
        // Approval turned on after the follow doesn't unfollow anyone.
        assert_eq!(status(&facts(Some(150))), Status::Accepted);
    }

    #[test]
    fn approval_waits_for_a_decision() {
        let mut f = facts(Some(50));
        assert_eq!(status(&f), Status::Pending);
        f.accepted_at = Some(120);
        assert_eq!(status(&f), Status::Accepted);
        f.rejected_at = Some(130);
        assert_eq!(status(&f), Status::Rejected);
    }

    #[test]
    fn decisions_before_the_follow_do_not_count() {
        let mut f = facts(Some(50));
        f.accepted_at = Some(90);
        assert_eq!(status(&f), Status::Pending);
        f.rejected_at = Some(80);
        assert_eq!(status(&f), Status::Pending);
    }
}
