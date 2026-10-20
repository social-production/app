//! Direct messages and group chats (the conversation DNA, ADR 0043). Shared by the messages
//! integrity zome (validation) and its coordinator (input checks before writing).
//!
//! Each chat is a clone of the conversation DNA seeded `<creator key>.<kind>.<nonce>`; the
//! uncloned base cell is everyone's inbox, where invitations to chats arrive encrypted.

use crate::limits::{text_ok, SHORT_TEXT_MAX};

/// A chat with one other person, or a named group (web's `conversations.kind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatKind {
    Direct,
    Group,
}

impl ChatKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Group => "group",
        }
    }
}

/// A group's name (web's `title`, required for a rename).
pub const TITLE_MAX: usize = SHORT_TEXT_MAX;
/// People in one chat, its creator included. web has no cap; this keeps one chat's invite
/// list small enough to read in one go.
pub const MEMBERS_MAX: usize = 50;
/// Bytes of an inbox link's tag (the encrypted invitation). Holochain caps tags at 1 KB.
pub const INBOX_TAG_MAX: usize = 900;
/// Shortest random part of a chat's seed: whoever knows the seed can find the network.
pub const NONCE_MIN: usize = 16;

/// `<creator>.<kind>.<nonce>` → the creator part and the kind. Any other seed (the base cell's
/// is empty) is the inbox network.
#[must_use]
pub fn chat_seed(seed: &str) -> Option<(&str, ChatKind)> {
    let mut parts = seed.splitn(3, '.');
    let creator = parts.next().filter(|c| !c.is_empty())?;
    let kind = match parts.next()? {
        "direct" => ChatKind::Direct,
        "group" => ChatKind::Group,
        _ => return None,
    };
    let nonce = parts.next()?;
    (nonce.len() >= NONCE_MIN).then_some((creator, kind))
}

/// A group name: some text after trimming, at most `TITLE_MAX` characters.
pub fn check_title(title: &str) -> Result<(), String> {
    if text_ok(title, TITLE_MAX) {
        Ok(())
    } else {
        Err(format!("a group name is 1–{TITLE_MAX} characters"))
    }
}

#[cfg(test)]
mod tests {
    use super::{chat_seed, check_title, ChatKind};

    #[test]
    fn chat_seeds_name_creator_and_kind() {
        let nonce = "abcdefghijklmnopqrstuv";
        assert_eq!(
            chat_seed(&format!("uhCAk.direct.{nonce}")),
            Some(("uhCAk", ChatKind::Direct))
        );
        assert_eq!(
            chat_seed(&format!("uhCAk.group.{nonce}")),
            Some(("uhCAk", ChatKind::Group))
        );
        assert_eq!(chat_seed(""), None, "the base cell is the inbox");
        assert_eq!(chat_seed("uhCAk.direct.short"), None, "nonces are long");
        assert_eq!(chat_seed(&format!("uhCAk.other.{nonce}")), None);
        assert_eq!(chat_seed(&format!(".group.{nonce}")), None);
    }

    #[test]
    fn group_names_need_text_within_the_cap() {
        assert!(check_title("Garden crew").is_ok());
        assert!(check_title("   ").is_err());
        assert!(check_title(&"x".repeat(201)).is_err());
    }
}
