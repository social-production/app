//! Channels and communities (ADR 0068), ported from web-backend `services/scopes.py`,
//! `scope_governance.py` and `utils/handles.py`. A scope is a space whose charter mode is
//! [`CHANNEL`] or [`COMMUNITY`]; its title is its handle and items name it in their tags.

use sp_core::handle::check_handle;

pub const CHANNEL: &str = "channel";
pub const COMMUNITY: &str = "community";
/// web-backend's `RESERVED_CHANNEL_SLUGS`: the platform's own channels.
pub const RESERVED: [&str; 2] = ["platform", "stewardship"];

/// The Platform channel (ADR 0095, web-backend `services/platform.py`): a fixed scope with
/// web's handle and description, written by whoever first follows it; readers take the
/// earliest. It has no manager, settings or proposals.
pub const PLATFORM: &str = "platform";
pub const PLATFORM_DESCRIPTION: &str = "The channel for the whole network. Follow it to see platform-wide projects, events, and governance.";

#[must_use]
pub fn is_platform(mode: &str, handle: &str) -> bool {
    mode == CHANNEL && handle == PLATFORM
}

/// The Platform channel's charter: exactly web's description, open, with no tags.
pub fn check_platform(description: &str, join_policy: &str, tags: &[String]) -> Result<(), String> {
    if description != PLATFORM_DESCRIPTION || join_policy != "open" || !tags.is_empty() {
        return Err("the Platform channel is fixed: web's description, open, no tags".into());
    }
    Ok(())
}

/// A community's join policy (web: `open` or `closed`); channels are always open.
pub const JOIN_POLICIES: [&str; 2] = ["open", "closed"];

#[must_use]
pub fn is_scope(mode: &str) -> bool {
    mode == CHANNEL || mode == COMMUNITY
}

/// web's `canonicalize_handle`: the slug two handles are compared by.
#[must_use]
pub fn slug(handle: &str) -> String {
    handle.trim().to_lowercase()
}

/// The tag an item carries for a scope: `#slug` for a channel, `@slug` for a community.
#[must_use]
pub fn tag(mode: &str, handle: &str) -> Option<String> {
    let sigil = match mode {
        CHANNEL => '#',
        COMMUNITY => '@',
        _ => return None,
    };
    Some(format!("{sigil}{}", slug(handle)))
}

/// A scope tag's mode and slug, if `tag` is one.
#[must_use]
pub fn parse_tag(tag: &str) -> Option<(&'static str, &str)> {
    let mode = match tag.chars().next()? {
        '#' => CHANNEL,
        '@' => COMMUNITY,
        _ => return None,
    };
    Some((mode, &tag[1..]))
}

/// A scope's charter: its handle (the title), its kind, and a community's join policy, which
/// the charter's subtype holds (channels: `"open"`).
pub fn check_scope(mode: &str, handle: &str, join_policy: &str) -> Result<(), String> {
    let label = if mode == CHANNEL {
        "Channel"
    } else {
        "Community"
    };
    if !is_scope(mode) {
        return Err(format!("a scope is a {CHANNEL} or a {COMMUNITY}"));
    }
    check_handle(handle).map_err(|e| e.replace("Username", &format!("{label} name")))?;
    if mode == CHANNEL && RESERVED.contains(&slug(handle).as_str()) {
        return Err("That channel name is reserved".into());
    }
    match (mode, join_policy) {
        (CHANNEL, "open") => Ok(()),
        (COMMUNITY, p) if JOIN_POLICIES.contains(&p) => Ok(()),
        _ => Err(format!(
            "a {label} can't use the join policy {join_policy:?}"
        )),
    }
}

/// web: only closed communities may be organizer-run; everything else is voted.
#[must_use]
pub fn may_be_organizer_run(mode: &str, join_policy: &str) -> bool {
    mode == COMMUNITY && join_policy == "closed"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_and_tags() {
        assert_eq!(slug(" Garden-Club "), "garden-club");
        assert_eq!(tag(CHANNEL, "Food").as_deref(), Some("#food"));
        assert_eq!(tag(COMMUNITY, "Brunswick").as_deref(), Some("@brunswick"));
        assert_eq!(tag("productive", "x"), None);
        assert_eq!(parse_tag("#food"), Some((CHANNEL, "food")));
        assert_eq!(parse_tag("@brunswick"), Some((COMMUNITY, "brunswick")));
        assert_eq!(parse_tag("food"), None);
    }

    #[test]
    fn scope_rules() {
        assert!(check_scope(CHANNEL, "Food", "open").is_ok());
        assert!(check_scope(CHANNEL, "Food", "closed").is_err());
        assert!(check_scope(CHANNEL, "Platform", "open").is_err());
        assert!(check_scope(COMMUNITY, "Brunswick", "closed").is_ok());
        let spaced = check_scope(COMMUNITY, "Two words", "open");
        assert!(
            spaced
                .as_ref()
                .is_err_and(|e| e.starts_with("Community name")),
            "{spaced:?}"
        );
        assert!(check_scope("event", "Food", "open").is_err());
        assert!(may_be_organizer_run(COMMUNITY, "closed"));
        assert!(is_platform(CHANNEL, "platform"));
        assert!(!is_platform(CHANNEL, "Platform") && !is_platform(COMMUNITY, "platform"));
        assert!(check_platform(PLATFORM_DESCRIPTION, "open", &[]).is_ok());
        assert!(check_platform("Mine now", "open", &[]).is_err());
        assert!(check_platform(PLATFORM_DESCRIPTION, "open", &["#food".into()]).is_err());
        assert!(!may_be_organizer_run(COMMUNITY, "open"));
        assert!(!may_be_organizer_run(CHANNEL, "open"));
    }
}
