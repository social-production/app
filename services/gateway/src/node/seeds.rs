//! Import a person's 32-byte seed (their 24-word phrase) into the keystore. The agent key
//! is the seed's ed25519 key, so the same phrase gives the same identity on every node and
//! in the native app (which imports it the same way through runtime-tauri).
//!
//! The import follows lair's `import_seed` contract as implemented in Holochain's
//! runtime-tauri (`holochain-conductor-runtime`, MIT OR Apache-2.0).

use anyhow::{anyhow, Context};
use holochain::prelude::AgentPubKey;
use lair_keystore_api::lair_store::LairEntryInfo;
use sha2::{Digest, Sha256};
use sodoken::crypto_box::{
    xsalsa_easy, xsalsa_keypair, XSALSA_MACBYTES, XSALSA_NONCEBYTES, XSALSA_PUBLICKEYBYTES,
};

use super::Node;

impl Node {
    /// Import `seed` (idempotent) and return its agent key.
    pub async fn import_seed(&self, seed: &[u8; 32]) -> anyhow::Result<AgentPubKey> {
        let tag = seed_tag(seed);
        let client = self.keystore().lair_client();
        if let Ok(LairEntryInfo::Seed { seed_info, .. }) =
            client.get_entry(tag.clone().into()).await
        {
            return Ok(AgentPubKey::from_raw_32(
                seed_info.ed25519_pub_key.0.to_vec(),
            ));
        }

        // A local sender keypair encrypts the seed to a lair-held recipient.
        let mut sender_pk = [0_u8; XSALSA_PUBLICKEYBYTES];
        let mut sender_sk = sodoken::SizedLockedArray::new()?;
        xsalsa_keypair(&mut sender_pk, &mut sender_sk.lock())?;
        let recipient = client
            .new_seed(uuid::Uuid::new_v4().to_string().into(), None, false)
            .await
            .context("create import recipient")?;

        let mut nonce = [0_u8; XSALSA_NONCEBYTES];
        sodoken::random::randombytes_buf(&mut nonce)?;
        let mut cipher = vec![0_u8; 32 + XSALSA_MACBYTES];
        xsalsa_easy(
            &mut cipher,
            seed,
            &nonce,
            &recipient.x25519_pub_key,
            &sender_sk.lock(),
        )?;

        let info = client
            .import_seed(
                sender_pk.into(),
                recipient.x25519_pub_key,
                None,
                nonce,
                cipher.into(),
                tag.into(),
                false,
            )
            .await
            .map_err(|e| anyhow!("import seed: {e}"))?;
        Ok(AgentPubKey::from_raw_32(info.ed25519_pub_key.0.to_vec()))
    }
}

/// Stable keystore tag for a seed, without storing the seed itself in the name.
fn seed_tag(seed: &[u8; 32]) -> String {
    let digest = Sha256::digest(seed);
    format!("person-seed-{}", crate::hex::encode(&digest[..16]))
}
