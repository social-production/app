//! Usernames, with web's rules (web/src/lib/utils/handles.ts, web-backend utils/handles.py):
//! 3–32 letters, numbers, hyphens or underscores; no spaces; no leading, trailing or repeated
//! separators. On a P2P network names aren't unique; the key tag tells two people apart.

pub const HANDLE_MIN: usize = 3;
pub const HANDLE_MAX: usize = 32;

/// `Err` holds web's wording for what is wrong.
pub fn check_handle(handle: &str) -> Result<(), String> {
    if handle.is_empty() {
        return Err("Username is required".into());
    }
    if handle.chars().any(char::is_whitespace) {
        return Err("Username cannot contain spaces".into());
    }
    let len = handle.chars().count();
    if !(HANDLE_MIN..=HANDLE_MAX).contains(&len) {
        return Err(format!(
            "Username must be {HANDLE_MIN}–{HANDLE_MAX} characters"
        ));
    }
    // letters/numbers in runs, joined by single '-' or '_'
    let runs_ok = handle
        .split(['-', '_'])
        .all(|run| !run.is_empty() && run.chars().all(|c| c.is_ascii_alphanumeric()));
    if runs_ok {
        Ok(())
    } else {
        Err("Use 3–32 characters: letters, numbers, hyphens, or underscores. No spaces, and no leading, trailing, or repeated separators.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::check_handle;

    #[test]
    fn web_rules() {
        for ok in ["Ann", "btstrp1", "big_bird", "the-best-damn", "a1b"] {
            assert_eq!(check_handle(ok), Ok(()), "{ok}");
        }
        for bad in [
            "",
            "ab",
            "Ann Tester",
            "-ann",
            "ann_",
            "a--b",
            "a_-b",
            "añn",
            &"x".repeat(33),
        ] {
            assert!(check_handle(bad).is_err(), "{bad}");
        }
    }
}
