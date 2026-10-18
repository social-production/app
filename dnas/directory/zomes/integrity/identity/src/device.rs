use hdi::prelude::*;

use crate::{invalid, DeviceLink};

/// Each side writes its own `DeviceLink`; the pairing counts once both exist (the
/// coordinator checks for the mirror). Votes are later counted per primary (M3).
pub fn validate_create(
    d: &DeviceLink,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    if d.primary == d.device {
        return invalid("a device cannot link to itself");
    }
    if action.author() != &d.primary && action.author() != &d.device {
        return invalid("only the two paired agents can write a device link");
    }
    Ok(ValidateCallbackResult::Valid)
}
