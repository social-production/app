//! The gateway's VAPID key (RFC 8292): a P-256 key pair made once and kept in `gateway.sqlite`
//! as PKCS#8. Browsers subscribe with its public key, and each push carries a short JWT signed
//! with it, so a push service only takes pushes from the gateway the browser subscribed with.
use anyhow::{anyhow, Context};
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING as ES256};

use crate::b64;

pub struct Vapid {
    key: EcdsaKeyPair,
    pkcs8: Vec<u8>,
    /// Who runs the gateway, for the push service (`sub`): a URL, not an email.
    contact: String,
}

impl Vapid {
    /// A new key pair.
    pub fn generate(contact: &str) -> anyhow::Result<Vapid> {
        let doc = EcdsaKeyPair::generate_pkcs8(&ES256, &SystemRandom::new())
            .map_err(|_| anyhow!("make a VAPID key"))?;
        Vapid::from_pkcs8(doc.as_ref(), contact)
    }

    /// The key pair saved earlier.
    pub fn from_pkcs8(pkcs8: &[u8], contact: &str) -> anyhow::Result<Vapid> {
        let key = EcdsaKeyPair::from_pkcs8(&ES256, pkcs8, &SystemRandom::new())
            .map_err(|_| anyhow!("read the VAPID key"))?;
        Ok(Vapid {
            key,
            pkcs8: pkcs8.to_vec(),
            contact: contact.into(),
        })
    }

    /// What to save.
    #[must_use]
    pub fn pkcs8(&self) -> &[u8] {
        &self.pkcs8
    }

    /// The public key browsers subscribe with (`applicationServerKey`), base64url.
    #[must_use]
    pub fn public_key(&self) -> String {
        b64(self.key.public_key().as_ref())
    }

    /// The `Authorization` header for a push to `endpoint`, valid for 12 hours from `now`.
    pub fn authorization(&self, endpoint: &str, now_secs: u64) -> anyhow::Result<String> {
        let audience = origin(endpoint)?;
        let header = b64(br#"{"typ":"JWT","alg":"ES256"}"#);
        let claims = serde_json::json!({ "aud": audience, "exp": now_secs + 12 * 3600, "sub": self.contact });
        let unsigned = format!("{header}.{}", b64(claims.to_string().as_bytes()));
        let signature = self
            .key
            .sign(&SystemRandom::new(), unsigned.as_bytes())
            .map_err(|_| anyhow!("sign the VAPID token"))?;
        Ok(format!(
            "vapid t={unsigned}.{}, k={}",
            b64(signature.as_ref()),
            self.public_key()
        ))
    }
}

/// `https://host[:port]` of a URL.
fn origin(url: &str) -> anyhow::Result<String> {
    let rest = url
        .strip_prefix("https://")
        .context("a push endpoint must be https")?;
    let host = rest.split('/').next().unwrap_or_default();
    if host.is_empty() {
        anyhow::bail!("a push endpoint needs a host");
    }
    Ok(format!("https://{host}"))
}

impl std::fmt::Debug for Vapid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vapid")
            .field("public_key", &self.public_key())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::signature::{UnparsedPublicKey, ECDSA_P256_SHA256_FIXED};

    #[test]
    fn the_token_verifies_with_the_public_key_and_names_the_push_service() -> anyhow::Result<()> {
        let vapid = Vapid::generate("https://ui.example")?;
        let again = Vapid::from_pkcs8(vapid.pkcs8(), "https://ui.example")?;
        assert_eq!(again.public_key(), vapid.public_key());
        let auth = vapid.authorization("https://fcm.googleapis.com/fcm/send/abc", 1_000)?;
        let token = auth
            .strip_prefix("vapid t=")
            .context("prefix")?
            .split(", k=")
            .next()
            .context("t")?;
        let (unsigned, signature) = token.rsplit_once('.').context("signature")?;
        let public = crate::unb64(&vapid.public_key())?;
        UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, public)
            .verify(unsigned.as_bytes(), &crate::unb64(signature)?)
            .map_err(|_| anyhow!("bad signature"))?;
        let claims = crate::unb64(unsigned.split('.').nth(1).context("claims")?)?;
        let claims: serde_json::Value = serde_json::from_slice(&claims)?;
        assert_eq!(claims["aud"], "https://fcm.googleapis.com");
        assert_eq!(claims["exp"], 1_000 + 12 * 3600);
        assert!(origin("http://insecure.example/x").is_err());
        Ok(())
    }
}
