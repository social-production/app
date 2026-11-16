//! Web Push for the gateway (ADR 0107): a browser that switched push on gives the gateway a
//! push subscription (an endpoint at its push service plus two keys). For each signal to an
//! agent the gateway hosts, it encrypts the notice to each of that agent's subscriptions
//! (RFC 8291) and sends it with a VAPID signature (RFC 8292).
//!
//! No Holochain here, so it builds and tests on its own: `cargo test -p push`.
mod encrypt;
mod pusher;
mod send;
mod store;
mod vapid;

pub use encrypt::encrypt;
pub use pusher::Pusher;
pub use send::{send, Outcome};
pub use store::Saved;
pub use vapid::Vapid;

use serde::{Deserialize, Serialize};

/// A browser's push subscription, as `PushSubscription.toJSON()` gives it.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Subscription {
    pub endpoint: String,
    pub keys: Keys,
}

/// The browser's P-256 public key and auth secret, base64url as the browser gives them.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Keys {
    pub p256dh: String,
    pub auth: String,
}

/// What the service worker shows: worded as the notification card is (ADR 0046), with the link
/// a tap opens. `category` is web's notification category, matched against the muted ones.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Notice {
    pub kind: String,
    pub category: String,
    pub title: String,
    pub body: String,
    pub link: String,
}

impl Notice {
    /// The notice a subscription that hides details gets ("Show what it's about" off).
    #[must_use]
    pub fn without_details(&self) -> Notice {
        Notice {
            title: "Something new on Social Production".into(),
            body: String::new(),
            ..self.clone()
        }
    }
}

pub(crate) fn b64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub(crate) fn unb64(text: &str) -> anyhow::Result<Vec<u8>> {
    use base64::Engine;
    let text = text.trim_end_matches('=');
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(text)?)
}
