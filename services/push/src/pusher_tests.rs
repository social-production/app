use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use super::*;
use crate::Keys;

/// One connection: each in-memory connection would be its own database.
async fn memory() -> anyhow::Result<SqlitePool> {
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?)
}

fn saved(endpoint: &str, muted: &[&str]) -> Saved {
    Saved {
        subscription: Subscription {
            endpoint: endpoint.into(),
            keys: Keys {
                p256dh: crate::b64(&[4u8; 65]),
                auth: crate::b64(&[1u8; 16]),
            },
        },
        muted: muted.iter().map(|m| (*m).to_string()).collect(),
        details: true,
    }
}

#[tokio::test]
async fn subscriptions_and_the_key_are_kept_per_agent() -> anyhow::Result<()> {
    let pool = memory().await?;
    let pusher = Pusher::open(pool.clone(), "https://ui.example").await?;
    let again = Pusher::open(pool.clone(), "https://ui.example").await?;
    assert_eq!(
        again.public_key(),
        pusher.public_key(),
        "the key is made once"
    );
    pusher
        .subscribe("ann", &saved("https://push.example/1", &[]))
        .await?;
    pusher
        .subscribe("ann", &saved("https://push.example/2", &["comments"]))
        .await?;
    pusher
        .subscribe("bob", &saved("https://push.example/3", &[]))
        .await?;
    assert!(pusher
        .subscribe("bob", &saved("http://push.example/4", &[]))
        .await
        .is_err());
    let ann = store::of_agent(&pool, "ann").await?;
    assert_eq!(ann.len(), 2);
    assert_eq!(ann[1].muted, vec!["comments".to_string()]);
    assert!(
        !pusher.unsubscribe("bob", "https://push.example/1").await?,
        "only your own"
    );
    assert!(pusher.unsubscribe("ann", "https://push.example/1").await?);
    assert_eq!(store::of_agent(&pool, "ann").await?.len(), 1);
    Ok(())
}

#[tokio::test]
async fn a_muted_category_sends_nothing() -> anyhow::Result<()> {
    let pool = memory().await?;
    let pusher = Pusher::open(pool, "https://ui.example").await?;
    pusher
        .subscribe("ann", &saved("https://push.example/1", &["comments"]))
        .await?;
    let notice = Notice {
        kind: "reply".into(),
        category: "comments".into(),
        title: "Replied to your project".into(),
        body: String::new(),
        link: "/".into(),
    };
    assert_eq!(pusher.notify("ann", &notice).await?, 0);
    assert_eq!(pusher.notify("nobody", &notice).await?, 0);
    Ok(())
}
