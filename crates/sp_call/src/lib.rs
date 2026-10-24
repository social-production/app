//! The line a finished voice call leaves in a direct chat (web's `log_call`, ADR 0051):
//! "Voice call · 4:12", "Missed call" / "No answer", or "Call declined". Shared by the messages
//! integrity zome (validation) and its coordinator (input checks and one line per call).
//! Not in `sp_core`: adding code there changes every DNA's hash.

use std::collections::BTreeSet;

/// web-backend `CALL_OUTCOMES`.
pub const OUTCOMES: [&str; 3] = ["completed", "missed", "declined"];
/// web-backend clamps a call's length to a day.
pub const SECONDS_MAX: u32 = 24 * 3600;
/// A call's id (web's `call_id` is a UUID; any short id the UI makes will do).
pub const ID_MAX: usize = 64;

/// A call line's fields are web's: a known outcome, an id, a length of at most a day, and
/// only a completed call has a length.
pub fn check_call(id: &str, outcome: &str, seconds: u32) -> Result<(), String> {
    if id.trim().is_empty() || id.len() > ID_MAX {
        return Err("A call needs a short id".into());
    }
    if !OUTCOMES.contains(&outcome) {
        return Err("Unknown call outcome".into());
    }
    if seconds > SECONDS_MAX {
        return Err("A call lasts at most a day".into());
    }
    if outcome != "completed" && seconds > 0 {
        return Err("Only a completed call has a length".into());
    }
    Ok(())
}

/// web's "either phone may report; the first wins": of `lines` (sorted oldest first), keep
/// the first for each call id.
pub fn first_per_call<T>(lines: Vec<T>, id: impl Fn(&T) -> &str) -> Vec<T> {
    let mut seen = BTreeSet::new();
    lines
        .into_iter()
        .filter(|line| seen.insert(id(line).to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_web_outcomes() {
        assert!(check_call("a1", "completed", 252).is_ok());
        assert!(check_call("a1", "missed", 0).is_ok());
        assert!(check_call("a1", "declined", 0).is_ok());
        assert!(check_call("a1", "completed", SECONDS_MAX).is_ok());
    }

    #[test]
    fn refuses_bad_calls() {
        assert!(check_call("", "completed", 1).is_err());
        assert!(check_call(&"x".repeat(ID_MAX + 1), "completed", 1).is_err());
        assert!(check_call("a1", "busy", 0).is_err());
        assert!(check_call("a1", "completed", SECONDS_MAX + 1).is_err());
        assert!(check_call("a1", "missed", 5).is_err());
    }

    #[test]
    fn keeps_the_first_line_per_call() {
        let lines = vec![("a", 1), ("b", 2), ("a", 3)];
        assert_eq!(first_per_call(lines, |l| l.0), vec![("a", 1), ("b", 2)]);
    }
}
