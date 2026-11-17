//! A profile's cover photo, and the notes that list someone's comments (Profiles and Comments
//! mockups §2–3; owner approved the directory reset 2026-11-16). A comment lives in its space's
//! own clone; the note is the directory's pointer to it, so a profile or a follower's Home can
//! show "commented" rows without joining every space. The cover is its own record, so reading
//! a profile (every avatar in every feed) never carries it.

use hdi::prelude::*;
use sp_core::limits::{text_ok, AVATAR_MIME_TYPES, NOTE_MAX};

use crate::invalid;

/// Covers are resized in the browser before upload; this is the hard cap.
pub const COVER_MAX_BYTES: usize = 512 * 1024;

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Cover {
    pub mime: String,
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

/// A channel's or community's picture or cover (ADR 0109), named by its charter and listing.
/// Content-addressed: the same image is one record.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct ScopeImage {
    pub mime: String,
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

/// "I commented in this space": the space's listing, the message in its clone, who it answers
/// (a reply), and the opening words, so a feed row needs no read of the space itself.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct CommentNote {
    pub space: ActionHash,
    pub message: ActionHash,
    pub reply_to: Option<AgentPubKey>,
    pub excerpt: String,
}

pub fn validate_cover(c: &Cover) -> ExternResult<ValidateCallbackResult> {
    if c.bytes.is_empty() || c.bytes.len() > COVER_MAX_BYTES {
        return invalid("a cover photo must be at most 512 KB");
    }
    if !AVATAR_MIME_TYPES.contains(&c.mime.as_str()) {
        return invalid("a cover photo must be JPEG, PNG or WebP");
    }
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_scope_image(i: &ScopeImage) -> ExternResult<ValidateCallbackResult> {
    if i.bytes.is_empty() || i.bytes.len() > COVER_MAX_BYTES {
        return invalid("a picture or cover must be at most 512 KB");
    }
    if !AVATAR_MIME_TYPES.contains(&i.mime.as_str()) {
        return invalid("a picture or cover must be JPEG, PNG or WebP");
    }
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_note(n: &CommentNote) -> ExternResult<ValidateCallbackResult> {
    if !text_ok(&n.excerpt, NOTE_MAX) {
        return invalid("a comment note needs its opening words, at most 280 characters");
    }
    Ok(ValidateCallbackResult::Valid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid(r: ExternResult<ValidateCallbackResult>) -> bool {
        matches!(r, Ok(ValidateCallbackResult::Valid))
    }

    #[test]
    fn covers_are_small_images() {
        let ok = Cover {
            mime: "image/jpeg".into(),
            bytes: vec![1; 1000],
        };
        assert!(valid(validate_cover(&ok)));
        let big = Cover {
            mime: "image/jpeg".into(),
            bytes: vec![1; COVER_MAX_BYTES + 1],
        };
        assert!(!valid(validate_cover(&big)));
        let gif = Cover {
            mime: "image/gif".into(),
            bytes: vec![1; 10],
        };
        assert!(!valid(validate_cover(&gif)));
    }

    #[test]
    fn scope_images_are_small_images() {
        let ok = ScopeImage {
            mime: "image/webp".into(),
            bytes: vec![1; 1000],
        };
        assert!(valid(validate_scope_image(&ok)));
        let empty = ScopeImage {
            mime: "image/png".into(),
            bytes: vec![],
        };
        assert!(!valid(validate_scope_image(&empty)));
        let big = ScopeImage {
            mime: "image/jpeg".into(),
            bytes: vec![1; COVER_MAX_BYTES + 1],
        };
        assert!(!valid(validate_scope_image(&big)));
    }

    #[test]
    fn notes_carry_short_text() {
        let hash = ActionHash::from_raw_36(vec![0; 36]);
        let note = |excerpt: &str| CommentNote {
            space: hash.clone(),
            message: hash.clone(),
            reply_to: None,
            excerpt: excerpt.into(),
        };
        assert!(valid(validate_note(&note("Tomatoes did best"))));
        assert!(!valid(validate_note(&note("  "))));
        assert!(!valid(validate_note(&note(&"x".repeat(NOTE_MAX + 1)))));
    }
}
