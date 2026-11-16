//! The push subscriptions and the VAPID key, in the gateway's own database (`gateway.sqlite`).
//! A subscription belongs to one hosted agent and one browser (its endpoint).
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use crate::{Keys, Subscription, Vapid};

const SCHEMA: &str = "
create table if not exists push_subscriptions (
  endpoint text primary key,
  agent text not null,
  p256dh text not null,
  auth text not null,
  muted text not null,
  details integer not null,
  created_at integer not null
);
create index if not exists push_subscriptions_agent on push_subscriptions(agent);
create table if not exists push_key (
  id integer primary key check (id = 1),
  pkcs8 blob not null
);";

/// A browser's subscription with its choices: the categories switched off on that device and
/// whether a push may say what it is about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saved {
    pub subscription: Subscription,
    pub muted: Vec<String>,
    pub details: bool,
}

pub(crate) async fn migrate(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::raw_sql(SCHEMA).execute(pool).await?;
    Ok(())
}

/// The saved VAPID key, or a new one saved now.
pub(crate) async fn vapid(pool: &SqlitePool, contact: &str) -> anyhow::Result<Vapid> {
    let row = sqlx::query("select pkcs8 from push_key where id = 1")
        .fetch_optional(pool)
        .await?;
    if let Some(row) = row {
        return Vapid::from_pkcs8(&row.get::<Vec<u8>, _>("pkcs8"), contact);
    }
    let vapid = Vapid::generate(contact)?;
    sqlx::query("insert into push_key (id, pkcs8) values (1, ?)")
        .bind(vapid.pkcs8())
        .execute(pool)
        .await?;
    Ok(vapid)
}

/// Save (or move to `agent`) the browser's subscription.
pub(crate) async fn save(
    pool: &SqlitePool,
    agent: &str,
    saved: &Saved,
    now: i64,
) -> anyhow::Result<()> {
    sqlx::query(
        "insert or replace into push_subscriptions
         (endpoint, agent, p256dh, auth, muted, details, created_at) values (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&saved.subscription.endpoint)
    .bind(agent)
    .bind(&saved.subscription.keys.p256dh)
    .bind(&saved.subscription.keys.auth)
    .bind(serde_json::to_string(&saved.muted)?)
    .bind(saved.details)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Forget a subscription; `agent` limits it to that agent's own (None: the push service said
/// it is gone).
pub(crate) async fn remove(
    pool: &SqlitePool,
    endpoint: &str,
    agent: Option<&str>,
) -> anyhow::Result<bool> {
    let done = match agent {
        Some(agent) => {
            sqlx::query("delete from push_subscriptions where endpoint = ? and agent = ?")
                .bind(endpoint)
                .bind(agent)
                .execute(pool)
                .await?
        }
        None => {
            sqlx::query("delete from push_subscriptions where endpoint = ?")
                .bind(endpoint)
                .execute(pool)
                .await?
        }
    };
    Ok(done.rows_affected() > 0)
}

/// Every browser `agent` switched push on in.
pub(crate) async fn of_agent(pool: &SqlitePool, agent: &str) -> anyhow::Result<Vec<Saved>> {
    let rows = sqlx::query(
        "select endpoint, p256dh, auth, muted, details from push_subscriptions where agent = ?",
    )
    .bind(agent)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|r| {
            Ok(Saved {
                subscription: Subscription {
                    endpoint: r.get("endpoint"),
                    keys: Keys {
                        p256dh: r.get("p256dh"),
                        auth: r.get("auth"),
                    },
                },
                muted: serde_json::from_str(&r.get::<String, _>("muted"))?,
                details: r.get("details"),
            })
        })
        .collect()
}
