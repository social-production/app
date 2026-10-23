//! Photos and files on chat messages (web's `message_attachments`, ADR 0049). Shared by the
//! messages integrity zome (validation) and its coordinator (input checks before writing).
//! Not in `sp_core`: adding code there changed the directory and project DNAs' hashes too.
//!
//! A file's bytes are split into chunks, each its own entry (Holochain caps an entry at a few
//! MB); the message lists every attachment with its chunks in order.

use sp_core::chat::{check_message, MESSAGE_MAX};
use sp_core::limits::optional_text_ok;

/// Bytes in one chunk entry. A photo (at most `PHOTO_MAX`) is always one chunk.
pub const CHUNK_MAX: usize = 1024 * 1024;
/// web-backend `MAX_IMAGE_BYTES`: photos are resized in the browser to fit.
pub const PHOTO_MAX: usize = 1024 * 1024;
/// web-backend `MAX_FILE_BYTES`.
pub const FILE_MAX: usize = 10 * 1024 * 1024;
/// web-backend `MAX_ATTACHMENTS`, per message.
pub const ATTACHMENTS_MAX: usize = 10;
/// web-backend `_safe_filename` keeps 200 characters.
pub const NAME_MAX: usize = 200;
/// A media type (web-backend `_MEDIA_TYPE`: two parts of at most 127 characters).
pub const MIME_MAX: usize = 255;
/// web-backend `IMAGE_TYPES`.
pub const PHOTO_TYPES: [&str; 3] = ["image/jpeg", "image/png", "image/webp"];

/// A photo (shown in the bubble) or any other file (a download).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Image,
    File,
}

impl Kind {
    /// web's `kind` values.
    #[must_use]
    pub fn parse(kind: &str) -> Option<Self> {
        match kind {
            "image" => Some(Self::Image),
            "file" => Some(Self::File),
            _ => None,
        }
    }
}

/// How many chunks hold `size` bytes.
#[must_use]
pub fn chunks_for(size: usize) -> usize {
    size.div_ceil(CHUNK_MAX)
}

/// One chunk: some bytes, at most `CHUNK_MAX`.
pub fn check_chunk(len: usize) -> Result<(), String> {
    if (1..=CHUNK_MAX).contains(&len) {
        Ok(())
    } else {
        Err(format!("a file part is 1–{CHUNK_MAX} bytes"))
    }
}

/// One attachment's description: web-backend `prepare_attachment`'s rules (no videos, photos
/// are JPEG/PNG/WebP up to 1 MB, files up to 10 MB), and the right number of chunks.
pub fn check_attachment(
    kind: &str,
    name: &str,
    mime: &str,
    size: usize,
    chunks: usize,
) -> Result<(), String> {
    let kind = Kind::parse(kind).ok_or("an attachment is a photo or a file")?;
    if name.trim().is_empty() || !optional_text_ok(name, NAME_MAX) {
        return Err(format!("a file name is 1–{NAME_MAX} characters"));
    }
    if mime.is_empty() || mime.len() > MIME_MAX || mime.split('/').count() != 2 {
        return Err("a file needs a media type".into());
    }
    if mime.starts_with("video/") {
        return Err("Videos are not supported yet".into());
    }
    let photo = PHOTO_TYPES.contains(&mime);
    match kind {
        Kind::Image if !photo => return Err("Use a JPEG, PNG, or WebP photo".into()),
        Kind::Image if size > PHOTO_MAX => return Err("Photos must be 1MB or smaller".into()),
        Kind::File if size > FILE_MAX => return Err("Files must be 10MB or smaller".into()),
        _ => {}
    }
    if size == 0 {
        return Err("The file is empty".into());
    }
    if chunks != chunks_for(size) {
        return Err("a file's parts don't match its size".into());
    }
    Ok(())
}

/// A message's text with `attachments` of them: text is optional beside attachments (web's
/// "photo with no caption"), required without.
pub fn check_body(body: &str, attachments: usize) -> Result<(), String> {
    if attachments > ATTACHMENTS_MAX {
        return Err(format!("You can attach up to {ATTACHMENTS_MAX} files"));
    }
    if attachments == 0 {
        return check_message(body);
    }
    if optional_text_ok(body, MESSAGE_MAX) {
        Ok(())
    } else {
        Err(format!("a message is at most {MESSAGE_MAX} characters"))
    }
}

/// What a list row or a quote shows for a message (web-backend `_message_preview`): its text,
/// else "Photo" for a photo, else the first file's name.
#[must_use]
pub fn preview_text(body: &str, first: Option<(&str, &str)>) -> String {
    if !body.trim().is_empty() {
        return body.to_string();
    }
    match first {
        Some(("image", _)) => "Photo".into(),
        Some((_, name)) => name.chars().take(NAME_MAX).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{check_attachment, check_body, check_chunk, chunks_for, preview_text, CHUNK_MAX};

    #[test]
    fn chunks_cover_the_size() {
        assert_eq!(chunks_for(1), 1);
        assert_eq!(chunks_for(CHUNK_MAX), 1);
        assert_eq!(chunks_for(CHUNK_MAX + 1), 2);
        assert_eq!(chunks_for(10 * CHUNK_MAX), 10);
        assert!(check_chunk(CHUNK_MAX).is_ok());
        assert!(check_chunk(0).is_err());
        assert!(check_chunk(CHUNK_MAX + 1).is_err());
    }

    #[test]
    fn attachments_follow_web_backends_rules() {
        assert!(check_attachment("image", "a.jpg", "image/jpeg", 500_000, 1).is_ok());
        assert!(check_attachment("image", "a.gif", "image/gif", 500, 1).is_err());
        assert!(check_attachment("image", "a.jpg", "image/jpeg", CHUNK_MAX + 1, 2).is_err());
        assert!(check_attachment("file", "a.pdf", "application/pdf", 3 * CHUNK_MAX, 3).is_ok());
        assert!(check_attachment("file", "a.pdf", "application/pdf", 3 * CHUNK_MAX, 2).is_err());
        assert!(check_attachment("file", "big", "application/zip", 11 * CHUNK_MAX, 11).is_err());
        assert!(check_attachment("file", "clip.mp4", "video/mp4", 10, 1).is_err());
        assert!(check_attachment("file", " ", "text/plain", 10, 1).is_err());
        assert!(check_attachment("file", "a", "nonsense", 10, 1).is_err());
        assert!(check_attachment("file", "a", "text/plain", 0, 0).is_err());
        assert!(check_attachment("other", "a", "text/plain", 10, 1).is_err());
    }

    #[test]
    fn text_is_optional_beside_attachments() {
        assert!(check_body("", 1).is_ok());
        assert!(check_body("  ", 0).is_err());
        assert!(check_body("hi", 0).is_ok());
        assert!(check_body("", 11).is_err());
    }

    #[test]
    fn previews_name_the_attachment_without_text() {
        assert_eq!(preview_text("hello", Some(("image", "a.jpg"))), "hello");
        assert_eq!(preview_text("", Some(("image", "a.jpg"))), "Photo");
        assert_eq!(preview_text(" ", Some(("file", "notes.pdf"))), "notes.pdf");
        assert_eq!(preview_text("", None), "");
    }
}
