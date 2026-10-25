use hdi::prelude::*;

use crate::{invalid, LinkTypes};

/// Follows (ADR 0054): the follower writes `Follows` (follower → followed) and its mirror
/// `Followers` (followed → follower); the person followed writes `FollowAccepts`,
/// `FollowRejects` (followed → follower) and `RequiresFollowApproval` (self → self).
pub fn validate_create_link(
    link_type: LinkTypes,
    action: &TypedAction<CreateLinkData>,
) -> ExternResult<ValidateCallbackResult> {
    let author = action.author();
    let base = action.base_address.clone().into_agent_pub_key();
    let target = action.target_address.clone().into_agent_pub_key();
    let (Some(base), Some(target)) = (base, target) else {
        return invalid("follow links join two agents");
    };
    let ok = match link_type {
        // Written from the author's own key to someone else's.
        LinkTypes::Follows | LinkTypes::FollowAccepts | LinkTypes::FollowRejects => {
            &base == author && &target != author
        }
        LinkTypes::Followers => &target == author && &base != author,
        LinkTypes::RequiresFollowApproval => &base == author && &target == author,
        _ => true,
    };
    if ok {
        Ok(ValidateCallbackResult::Valid)
    } else {
        invalid(
            "follow links are written by the follower or the person followed, never about yourself",
        )
    }
}
