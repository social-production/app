//! What the gateway holds: the database, the VAPID key and an HTTP client. Routes save and
//! remove subscriptions; a signal to a hosted agent becomes `notify`.
use sqlx::sqlite::SqlitePool;

use crate::{send, store, Notice, Outcome, Saved, Subscription, Vapid};

#[derive(Debug)]
pub struct Pusher {
    pool: SqlitePool,
    vapid: Vapid,
    http: reqwest::Client,
}

impl Pusher {
    /// Make the tables and load (or make) the VAPID key. `contact` is a URL for the push
    /// services (the UI's origin).
    pub async fn open(pool: SqlitePool, contact: &str) -> anyhow::Result<Pusher> {
        store::migrate(&pool).await?;
        let vapid = store::vapid(&pool, contact).await?;
        Ok(Pusher {
            pool,
            vapid,
            http: reqwest::Client::new(),
        })
    }

    /// The key browsers subscribe with (`applicationServerKey`).
    #[must_use]
    pub fn public_key(&self) -> String {
        self.vapid.public_key()
    }

    /// `agent` switched push on in a browser (or changed its choices there).
    pub async fn subscribe(&self, agent: &str, saved: &Saved) -> anyhow::Result<()> {
        anyhow::ensure!(
            saved.subscription.endpoint.starts_with("https://"),
            "a push endpoint must be https"
        );
        anyhow::ensure!(
            saved.subscription.endpoint.len() <= 1024,
            "a push endpoint is too long"
        );
        crate::unb64(&saved.subscription.keys.p256dh)?;
        crate::unb64(&saved.subscription.keys.auth)?;
        store::save(&self.pool, agent, saved, now()).await
    }

    /// `agent` switched push off in the browser with `endpoint`, or signed out there.
    pub async fn unsubscribe(&self, agent: &str, endpoint: &str) -> anyhow::Result<bool> {
        store::remove(&self.pool, endpoint, Some(agent)).await
    }

    /// The agents to listen for: those with push on somewhere.
    pub async fn agents(&self) -> anyhow::Result<Vec<String>> {
        store::agents(&self.pool).await
    }

    /// Push `notice` to every browser of `agent` that has its category on. Subscriptions the
    /// push service calls gone are forgotten. Returns how many pushes were taken.
    pub async fn notify(&self, agent: &str, notice: &Notice) -> anyhow::Result<usize> {
        let mut sent = 0;
        for saved in store::of_agent(&self.pool, agent).await? {
            if saved.muted.contains(&notice.category) {
                continue;
            }
            let notice = if saved.details {
                notice.clone()
            } else {
                notice.without_details()
            };
            match self.push(&saved.subscription, &notice).await {
                Outcome::Sent => sent += 1,
                Outcome::Gone => {
                    store::remove(&self.pool, &saved.subscription.endpoint, None).await?;
                }
                Outcome::Failed(why) => tracing::warn!(%why, "push failed"),
            }
        }
        Ok(sent)
    }

    async fn push(&self, subscription: &Subscription, notice: &Notice) -> Outcome {
        send(
            &self.http,
            &self.vapid,
            subscription,
            notice,
            u64::try_from(now()).unwrap_or(0),
        )
        .await
    }
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

#[cfg(test)]
#[path = "pusher_tests.rs"]
mod tests;
