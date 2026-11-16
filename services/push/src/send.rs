//! Sending one push to one subscription, and what the gateway does with the answer.
use std::time::Duration;

use crate::{encrypt, Notice, Subscription, Vapid};

/// What became of a push.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The push service took it.
    Sent,
    /// The subscription is gone (404 or 410): the gateway forgets it.
    Gone,
    /// Anything else (rate limits, the push service down): try again with the next notice.
    Failed(String),
}

/// How long a push service keeps a push for a browser that is offline.
const TTL_SECS: u64 = 24 * 3600;

/// Encrypt `notice` to `subscription` and post it to its push service.
pub async fn send(
    http: &reqwest::Client,
    vapid: &Vapid,
    subscription: &Subscription,
    notice: &Notice,
    now_secs: u64,
) -> Outcome {
    match post(http, vapid, subscription, notice, now_secs).await {
        Ok(status) => outcome(status),
        Err(e) => Outcome::Failed(format!("{e:#}")),
    }
}

async fn post(
    http: &reqwest::Client,
    vapid: &Vapid,
    subscription: &Subscription,
    notice: &Notice,
    now_secs: u64,
) -> anyhow::Result<u16> {
    let body = encrypt(&subscription.keys, &serde_json::to_vec(notice)?)?;
    let response = http
        .post(&subscription.endpoint)
        .timeout(Duration::from_secs(15))
        .header(
            "Authorization",
            vapid.authorization(&subscription.endpoint, now_secs)?,
        )
        .header("Content-Encoding", "aes128gcm")
        .header("Content-Type", "application/octet-stream")
        .header("TTL", TTL_SECS.to_string())
        .header("Urgency", "normal")
        .body(body)
        .send()
        .await?;
    Ok(response.status().as_u16())
}

fn outcome(status: u16) -> Outcome {
    match status {
        200..=299 => Outcome::Sent,
        404 | 410 => Outcome::Gone,
        other => Outcome::Failed(format!("push service answered {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gone_subscription_is_forgotten_and_others_kept() {
        assert_eq!(outcome(201), Outcome::Sent);
        assert_eq!(outcome(410), Outcome::Gone);
        assert_eq!(outcome(404), Outcome::Gone);
        assert!(matches!(outcome(429), Outcome::Failed(_)));
    }

    #[test]
    fn hiding_details_keeps_the_link_and_category() {
        let notice = Notice {
            kind: "reply".into(),
            category: "comments".into(),
            title: "Replied to your project".into(),
            body: "Soil arrives Saturday".into(),
            link: "/projects/x?tab=chat".into(),
        };
        let bare = notice.without_details();
        assert_eq!(bare.title, "Something new on Social Production");
        assert!(bare.body.is_empty());
        assert_eq!(
            (bare.link.as_str(), bare.category.as_str()),
            (notice.link.as_str(), "comments")
        );
    }
}
