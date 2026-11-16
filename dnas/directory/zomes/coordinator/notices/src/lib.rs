//! Notices (ADR 0108): when a write makes a card for someone, the writer's app also tells them
//! at once with a remote signal. The signal is a nudge, not a record: the Notifications page is
//! still built from the chains (ADR 0046), so a missed signal loses nothing. The receiver's
//! `recv_remote_signal` re-emits it to its own app, where the seed node turns it into a Web Push
//! and the desktop app into an OS notification.
//!
//! Only the directory DNA has this zome: everyone shares that one network, so each person needs
//! a single grant for it. Coordinator only, so no network reset.

use hdk::prelude::*;

/// The grant's tag, so `enable_notices` finds the one it made before.
const TAG: &str = "notices";
/// The most people one notice goes to (web's `MAX_REVIEW_NOTICES`).
pub const MAX_RECIPIENTS: usize = 100;
const MAX_KIND: usize = 40;
const MAX_TITLE: usize = 200;
const MAX_BODY: usize = 500;
const MAX_LINK: usize = 300;

/// One notice, worded as its card is (`push::Notice` on the seed node).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub kind: String,
    pub category: String,
    pub title: String,
    pub body: String,
    /// An in-app path (`/projects/…`); tapping the notification opens it.
    pub link: String,
}

/// What travels and what the receiver's app hears: `{ notice: … }`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Envelope {
    pub notice: Notice,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendInput {
    pub to: Vec<AgentPubKey>,
    pub notice: Notice,
}

/// Why a notice is refused, if it is (both on sending and on receiving: a peer's app may not
/// be ours).
#[must_use]
pub fn problem(n: &Notice) -> Option<&'static str> {
    let fits = |s: &str, max: usize| s.chars().count() <= max;
    if n.kind.is_empty() || !fits(&n.kind, MAX_KIND) || !fits(&n.category, MAX_KIND) {
        return Some("A notice needs a short kind and category.");
    }
    if n.title.trim().is_empty() || !fits(&n.title, MAX_TITLE) || !fits(&n.body, MAX_BODY) {
        return Some("A notice needs a title, and its text must be short.");
    }
    if !n.link.starts_with('/') || n.link.starts_with("//") || !fits(&n.link, MAX_LINK) {
        return Some("A notice's link must be a path in the app.");
    }
    None
}

fn guest(reason: &str) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(reason.into()))
}

/// Let anyone's app deliver notices to this one: an unrestricted grant for
/// `recv_remote_signal` only. The app calls it as it starts; it writes once.
#[hdk_extern]
pub fn enable_notices(_: ()) -> ExternResult<bool> {
    if has_grant()? {
        return Ok(false);
    }
    let mut functions = HashSet::new();
    functions.insert((zome_info()?.name, "recv_remote_signal".into()));
    create_cap_grant(CapGrant::new_zome_call_grant(
        TAG.into(),
        GrantConstraint::Unrestricted,
        GrantedFunctions::Listed(functions),
    ))?;
    Ok(true)
}

fn has_grant() -> ExternResult<bool> {
    let filter = ChainQueryFilter::new()
        .entry_type(EntryType::CapGrant)
        .include_entries(true);
    Ok(query(filter)?.iter().any(|record| {
        matches!(
            record.entry().as_option(),
            Some(Entry::CapGrant(grant)) if grant.tag == TAG
        )
    }))
}

/// Tell `to` about something new (you are left out). Fire and forget: whoever is offline
/// finds the card when they next open the app.
#[hdk_extern]
pub fn send_notice(input: SendInput) -> ExternResult<usize> {
    if let Some(reason) = problem(&input.notice) {
        return Err(guest(reason));
    }
    let me = agent_info()?.agent_initial_pubkey;
    let mut to: Vec<AgentPubKey> = Vec::new();
    for agent in input.to {
        if agent != me && !to.contains(&agent) {
            to.push(agent);
        }
    }
    to.truncate(MAX_RECIPIENTS);
    if to.is_empty() {
        return Ok(0);
    }
    let count = to.len();
    send_remote_signal(
        Envelope {
            notice: input.notice,
        },
        to,
    )?;
    Ok(count)
}

/// A notice from someone's app: pass it to this person's own app if it is well formed.
#[hdk_extern]
pub fn recv_remote_signal(envelope: Envelope) -> ExternResult<()> {
    // The call's payload is the sent value itself, so a signal that isn't a notice fails to
    // decode and never gets here.
    if problem(&envelope.notice).is_some() {
        return Ok(());
    }
    emit_signal(&envelope)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notice() -> Notice {
        Notice {
            kind: "reply".into(),
            category: "comments".into(),
            title: "Ann replied to your comment".into(),
            body: "Looks good".into(),
            link: "/projects/x?tab=chat".into(),
        }
    }

    #[test]
    fn a_card_shaped_notice_passes() {
        assert_eq!(problem(&notice()), None);
    }

    #[test]
    fn links_stay_inside_the_app() {
        for link in ["https://evil.example", "//evil.example", "projects/x"] {
            let n = Notice {
                link: link.into(),
                ..notice()
            };
            assert!(problem(&n).is_some(), "{link}");
        }
    }

    #[test]
    fn long_or_empty_text_is_refused() {
        let empty = Notice {
            title: " ".into(),
            ..notice()
        };
        let long = Notice {
            body: "x".repeat(MAX_BODY + 1),
            ..notice()
        };
        assert!(problem(&empty).is_some());
        assert!(problem(&long).is_some());
    }
}
