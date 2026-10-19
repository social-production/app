//! Project charter fields, as on the current site (web-backend `ProjectCreateRequest`).
//! Shared by the project DNA (charter) and the directory DNA (listing).

use crate::limits::{optional_text_ok, text_ok, LONG_TEXT_MAX};

pub const TITLE_MAX: usize = 200;
pub const DESCRIPTION_MAX: usize = LONG_TEXT_MAX;
/// The listing carries a short summary of the description for feed cards.
pub const SUMMARY_MAX: usize = 500;
pub const LOCATION_LABEL_MAX: usize = 160;
pub const PHASE_MAX: usize = 60;
pub const TAGS_MAX: usize = 10;
pub const TAG_MAX: usize = 40;
pub const MODES: [&str; 3] = ["productive", "collective-service", "personal-service"];
pub const SUBTYPES: [&str; 3] = ["standard", "software", "asset-management"];

/// The fields every charter and listing share.
#[derive(Clone, Copy, Debug)]
pub struct CharterFields<'a> {
    pub title: &'a str,
    pub mode: &'a str,
    pub subtype: &'a str,
    pub location_label: &'a str,
    pub phase: &'a str,
    pub tags: &'a [String],
}

/// `Err` names the first field that breaks the rules.
pub fn check_fields(f: &CharterFields<'_>) -> Result<(), String> {
    if !text_ok(f.title, TITLE_MAX) {
        return Err(format!("title must be 1–{TITLE_MAX} characters"));
    }
    if !MODES.contains(&f.mode) {
        return Err(format!("unknown project mode {:?}", f.mode));
    }
    if !SUBTYPES.contains(&f.subtype) {
        return Err(format!("unknown project type {:?}", f.subtype));
    }
    if !optional_text_ok(f.location_label, LOCATION_LABEL_MAX) {
        return Err(format!(
            "location must be at most {LOCATION_LABEL_MAX} characters"
        ));
    }
    if !optional_text_ok(f.phase, PHASE_MAX) {
        return Err(format!("phase must be at most {PHASE_MAX} characters"));
    }
    if f.tags.len() > TAGS_MAX {
        return Err(format!("at most {TAGS_MAX} tags"));
    }
    if f.tags.iter().any(|t| !text_ok(t, TAG_MAX)) {
        return Err(format!("tags must be 1–{TAG_MAX} characters"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields<'a>(title: &'a str, mode: &'a str, tags: &'a [String]) -> CharterFields<'a> {
        CharterFields {
            title,
            mode,
            subtype: "standard",
            location_label: "",
            phase: "",
            tags,
        }
    }

    #[test]
    fn accepts_web_values() {
        let tags = vec!["food".to_string()];
        assert_eq!(
            check_fields(&fields("Community garden", "productive", &tags)),
            Ok(())
        );
    }

    #[test]
    fn rejects_bad_fields() {
        assert!(check_fields(&fields(" ", "productive", &[])).is_err());
        assert!(check_fields(&fields("Garden", "commercial", &[])).is_err());
        let many: Vec<String> = (0..11).map(|i| i.to_string()).collect();
        assert!(check_fields(&fields("Garden", "productive", &many)).is_err());
        assert!(check_fields(&fields("Garden", "productive", &[String::new()])).is_err());
    }
}
