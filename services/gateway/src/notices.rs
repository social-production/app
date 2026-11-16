//! Push notifications (ADR 0108): the seed node listens to the signals of each person with push
//! on, and a signal carrying `{ notice: … }` becomes a Web Push to their browsers.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use holo_hash::AgentPubKeyB64;
use holochain::prelude::*;
use holochain_types::signal::Signal;
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;

use crate::node::{app_id, Node};

#[derive(Debug, Deserialize)]
struct Envelope {
    notice: push::Notice,
}

/// The notice a signal carries, if it is one (other signals are the UI's own).
pub fn notice_of(signal: &Signal) -> Option<push::Notice> {
    let Signal::App { signal, .. } = signal else {
        return None;
    };
    let envelope: Envelope = signal.clone().into_inner().decode().ok()?;
    Some(envelope.notice)
}

/// The agents being listened to, so each gets one listener.
#[derive(Clone, Debug, Default)]
pub struct Listeners(Arc<Mutex<HashSet<String>>>);

impl Listeners {
    /// Listen for everyone with push on (at start).
    pub async fn start_all(&self, node: &Node, pusher: &Arc<push::Pusher>) {
        match pusher.agents().await {
            Ok(agents) => {
                for agent in agents {
                    self.listen(node, pusher, &agent);
                }
            }
            Err(e) => tracing::warn!(error = %e, "push agents"),
        }
    }

    /// Start listening for `agent` unless already.
    pub fn listen(&self, node: &Node, pusher: &Arc<push::Pusher>, agent: &str) {
        let Ok(key) = AgentPubKeyB64::from_b64_str(agent).map(AgentPubKey::from) else {
            return;
        };
        if !self
            .0
            .lock()
            .is_ok_and(|mut set| set.insert(agent.to_string()))
        {
            return;
        }
        let (node, pusher, set) = (node.clone(), pusher.clone(), self.0.clone());
        let agent = agent.to_string();
        tokio::spawn(async move {
            Box::pin(listen_until_gone(&node, &pusher, &key, &agent)).await;
            if let Ok(mut set) = set.lock() {
                set.remove(&agent);
            }
        });
    }
}

/// Hear the person's signals until their app is no longer hosted here (moved to a desktop or
/// a new network); the channel closes when the app is removed.
async fn listen_until_gone(node: &Node, pusher: &push::Pusher, key: &AgentPubKey, agent: &str) {
    loop {
        let mut signals = node.app_signals(&app_id(key, &node.happ_version));
        loop {
            match signals.recv().await {
                Ok(signal) => {
                    if let Some(notice) = notice_of(&signal) {
                        if let Err(e) = pusher.notify(agent, &notice).await {
                            tracing::warn!(error = %e, "push notify");
                        }
                    }
                }
                Err(RecvError::Lagged(missed)) => tracing::warn!(missed, "push signals lagged"),
                Err(RecvError::Closed) => break,
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
        if !node.hosts(key).await.unwrap_or(true) {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_signal(value: &serde_json::Value) -> Signal {
        Signal::App {
            cell_id: CellId::new(
                DnaHash::from_raw_36(vec![1; 36]),
                AgentPubKey::from_raw_36(vec![2; 36]),
            ),
            zome_name: "notices".into(),
            signal: AppSignal::new(ExternIO::encode(value).expect("encode")),
        }
    }

    #[test]
    fn only_a_notice_signal_becomes_a_push() {
        let notice = serde_json::json!({ "notice": {
            "kind": "comment", "category": "comments", "title": "Ann replied",
            "body": "Looks good", "link": "/projects/x"
        }});
        let found = notice_of(&app_signal(&notice)).expect("a notice");
        assert_eq!(found.category, "comments");
        assert_eq!(found.link, "/projects/x");
        assert!(notice_of(&app_signal(&serde_json::json!({ "typing": true }))).is_none());
    }
}
