//! The conductor lists every lair entry in one message on each app install (ADR 0034). With
//! upstream's 8 KiB frame a node stopped installing after ~55 people; the vendored lair patch
//! raises that to 64 KiB. This proves a node with 300 people can still list its keys.

use std::sync::{Arc, Mutex};

#[tokio::test(flavor = "multi_thread")]
async fn three_hundred_people_fit_in_one_key_listing() {
    let dir = std::env::temp_dir().join(format!("sp-lair-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let passphrase = Arc::new(Mutex::new(b"test-passphrase".to_vec().into()));
    let keystore = holochain_keystore::lair_keystore::spawn_lair_keystore_in_proc(
        &dir.join("lair-keystore-config.yaml"),
        passphrase,
    )
    .await
    .expect("keystore");
    let client = keystore.lair_client();
    for i in 0..300 {
        // Same tag shape as node/seeds.rs: "p-" + 24 hex characters.
        let tag = format!("p-{i:024x}");
        client
            .new_seed(tag.into(), None, false)
            .await
            .expect("new seed");
    }
    let listed = keystore
        .list_public_keys()
        .await
        .expect("list fits in one frame");
    assert!(listed.len() >= 300);
    let _ = std::fs::remove_dir_all(&dir);
}
