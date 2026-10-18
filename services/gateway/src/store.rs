//! The gateway's own small database (`SQLite` on the data volume). It holds only what the
//! gateway needs to host people: which passkeys and device signing keys belong to which
//! hosted agent. No names, emails or passwords; everything else lives in Holochain.

use std::path::Path;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::Row;

const SCHEMA: &str = "
create table if not exists persons (
  agent text primary key,
  created_at integer not null,
  last_seen integer not null
);
create table if not exists passkeys (
  cred_id text primary key,
  agent text not null references persons(agent),
  passkey text not null
);
create table if not exists signing_keys (
  signing_key text primary key,
  agent text not null references persons(agent),
  created_at integer not null
);";

#[derive(Clone, Debug)]
pub struct Store {
    pool: SqlitePool,
}

impl Store {
    pub async fn open(data_dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let options = SqliteConnectOptions::new()
            .filename(data_dir.join("gateway.sqlite"))
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
        sqlx::raw_sql(SCHEMA).execute(&pool).await?;
        // M1.1: device secrets. Older databases lack the column; adding it twice fails harmlessly.
        let _ = sqlx::query("alter table signing_keys add column secret_hash text")
            .execute(&pool)
            .await;
        Ok(Self { pool })
    }

    pub async fn person_exists(&self, agent: &str) -> anyhow::Result<bool> {
        let row = sqlx::query("select 1 from persons where agent = ?")
            .bind(agent)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.is_some())
    }

    pub async fn upsert_person(&self, agent: &str) -> anyhow::Result<()> {
        let now = now();
        sqlx::query(
            "insert into persons (agent, created_at, last_seen) values (?, ?, ?)
             on conflict(agent) do update set last_seen = excluded.last_seen",
        )
        .bind(agent)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_passkey(
        &self,
        cred_id: &str,
        agent: &str,
        passkey: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "insert into passkeys (cred_id, agent, passkey) values (?, ?, ?)
             on conflict(cred_id) do update set passkey = excluded.passkey",
        )
        .bind(cred_id)
        .bind(agent)
        .bind(passkey)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// (agent, passkey json) for a credential id.
    pub async fn passkey(&self, cred_id: &str) -> anyhow::Result<Option<(String, String)>> {
        let row = sqlx::query("select agent, passkey from passkeys where cred_id = ?")
            .bind(cred_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| (r.get("agent"), r.get("passkey"))))
    }

    /// Remember a device's signing key and (hashed) device secret for its person.
    pub async fn save_signing_key(
        &self,
        signing_key: &str,
        agent: &str,
        device_secret: &[u8],
    ) -> anyhow::Result<()> {
        sqlx::query(
            "insert or replace into signing_keys (signing_key, agent, created_at, secret_hash)
             values (?, ?, ?, ?)",
        )
        .bind(signing_key)
        .bind(agent)
        .bind(now())
        .bind(secret_hash(device_secret))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// The person behind a device, if the device secret matches.
    pub async fn agent_for_device(
        &self,
        signing_key: &str,
        device_secret: &[u8],
    ) -> anyhow::Result<Option<String>> {
        let row =
            sqlx::query("select agent from signing_keys where signing_key = ? and secret_hash = ?")
                .bind(signing_key)
                .bind(secret_hash(device_secret))
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|r| r.get("agent")))
    }

    pub async fn agent_for_signing_key(&self, signing_key: &str) -> anyhow::Result<Option<String>> {
        let row = sqlx::query("select agent from signing_keys where signing_key = ?")
            .bind(signing_key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.get("agent")))
    }
}

fn secret_hash(secret: &[u8]) -> String {
    use sha2::Digest;
    crate::hex::encode(&sha2::Sha256::digest(secret))
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}
