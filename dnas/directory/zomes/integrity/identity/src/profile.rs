use hdi::prelude::*;
use sp_core::limits::{
    optional_text_ok, text_ok, AVATAR_MAX_BYTES, AVATAR_MIME_TYPES, BIO_MAX, DISPLAY_NAME_MAX,
    HOME_AREA_MAX,
};

use crate::{invalid, Profile};

pub fn validate_fields(p: &Profile) -> ExternResult<ValidateCallbackResult> {
    if !text_ok(&p.name, DISPLAY_NAME_MAX) {
        return invalid("name must be 1-60 characters");
    }
    if !optional_text_ok(&p.bio, BIO_MAX) {
        return invalid("bio must be at most 1000 characters");
    }
    if !optional_text_ok(&p.home_area, HOME_AREA_MAX) {
        return invalid("home area must be at most 100 characters");
    }
    if let Some(avatar) = &p.avatar {
        if avatar.bytes.len() > AVATAR_MAX_BYTES {
            return invalid("avatar must be at most 256 KB");
        }
        if !AVATAR_MIME_TYPES.contains(&avatar.mime.as_str()) {
            return invalid("avatar must be JPEG, PNG or WebP");
        }
    }
    Ok(ValidateCallbackResult::Valid)
}

/// Only the person who wrote a profile can update it.
pub fn validate_update(
    p: &Profile,
    action: &TypedAction<UpdateData>,
) -> ExternResult<ValidateCallbackResult> {
    let original = must_get_action(action.original_action_address.clone())?;
    if original.action().author() != action.author() {
        return invalid("only the profile's owner can update it");
    }
    validate_fields(p)
}
