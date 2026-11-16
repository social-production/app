//! The push payload's encryption (RFC 8291, "aes128gcm" content coding of RFC 8188): only the
//! browser that subscribed can read it; the push service sees bytes.
use anyhow::{anyhow, ensure};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_128_GCM};
use ring::agreement::{agree_ephemeral, EphemeralPrivateKey, UnparsedPublicKey, ECDH_P256};
use ring::hkdf::{KeyType, Salt, HKDF_SHA256};
use ring::rand::{SecureRandom, SystemRandom};

use crate::{unb64, Keys};

/// Record size written in the header; one record holds the whole payload.
const RECORD_SIZE: u32 = 4096;

/// The body of a push to the browser with `keys`: salt, record size, the gateway's one-time
/// public key, then the sealed `plaintext`.
pub fn encrypt(keys: &Keys, plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let ua_public = unb64(&keys.p256dh)?;
    let auth = unb64(&keys.auth)?;
    ensure!(
        ua_public.len() == 65 && auth.len() == 16,
        "a subscription's keys have the wrong size"
    );
    ensure!(
        plaintext.len() + 17 + 86 <= RECORD_SIZE as usize,
        "a push payload must stay under 4 kB"
    );
    let rng = SystemRandom::new();
    let private =
        EphemeralPrivateKey::generate(&ECDH_P256, &rng).map_err(|_| anyhow!("make a key"))?;
    let as_public = private
        .compute_public_key()
        .map_err(|_| anyhow!("public key"))?;
    let mut salt = [0u8; 16];
    rng.fill(&mut salt).map_err(|_| anyhow!("random salt"))?;
    let shared = agree_ephemeral(
        private,
        &UnparsedPublicKey::new(&ECDH_P256, &ua_public),
        <[u8]>::to_vec,
    )
    .map_err(|_| anyhow!("the subscription's public key is not a P-256 point"))?;
    let sealed = seal(
        &shared,
        &auth,
        &ua_public,
        as_public.as_ref(),
        &salt,
        plaintext,
    )?;
    let mut body = Vec::with_capacity(16 + 4 + 1 + 65 + sealed.len());
    body.extend_from_slice(&salt);
    body.extend_from_slice(&RECORD_SIZE.to_be_bytes());
    body.push(u8::try_from(as_public.as_ref().len())?);
    body.extend_from_slice(as_public.as_ref());
    body.extend_from_slice(&sealed);
    Ok(body)
}

/// The keys from the shared secret, then AES-128-GCM over the payload and its last-record
/// delimiter (0x02). The browser runs the same steps with its own private key.
pub(crate) fn seal(
    shared: &[u8],
    auth: &[u8],
    ua_public: &[u8],
    as_public: &[u8],
    salt: &[u8],
    plaintext: &[u8],
) -> anyhow::Result<Vec<u8>> {
    let (cek, nonce) = content_keys(shared, auth, ua_public, as_public, salt)?;
    let key = LessSafeKey::new(UnboundKey::new(&AES_128_GCM, &cek).map_err(|_| anyhow!("key"))?);
    let mut record = plaintext.to_vec();
    record.push(2);
    key.seal_in_place_append_tag(
        Nonce::assume_unique_for_key(nonce),
        Aad::empty(),
        &mut record,
    )
    .map_err(|_| anyhow!("seal"))?;
    Ok(record)
}

/// RFC 8291 section 3.4 and RFC 8188 section 2.2: the content key and the nonce.
pub(crate) fn content_keys(
    shared: &[u8],
    auth: &[u8],
    ua_public: &[u8],
    as_public: &[u8],
    salt: &[u8],
) -> anyhow::Result<([u8; 16], [u8; 12])> {
    let key_info = [b"WebPush: info\0".as_slice(), ua_public, as_public].concat();
    let ikm = expand(auth, shared, &key_info, 32)?;
    let cek = expand(salt, &ikm, b"Content-Encoding: aes128gcm\0", 16)?;
    let nonce = expand(salt, &ikm, b"Content-Encoding: nonce\0", 12)?;
    let mut keys = ([0u8; 16], [0u8; 12]);
    keys.0.copy_from_slice(&cek);
    keys.1.copy_from_slice(&nonce);
    Ok(keys)
}

struct Len(usize);
impl KeyType for Len {
    fn len(&self) -> usize {
        self.0
    }
}

/// HKDF-SHA-256: extract with `salt`, then expand `info` to `len` bytes.
fn expand(salt: &[u8], ikm: &[u8], info: &[u8], len: usize) -> anyhow::Result<Vec<u8>> {
    let prk = Salt::new(HKDF_SHA256, salt).extract(ikm);
    let info = [info];
    let okm = prk.expand(&info, Len(len)).map_err(|_| anyhow!("hkdf"))?;
    let mut out = vec![0u8; len];
    okm.fill(&mut out).map_err(|_| anyhow!("hkdf"))?;
    Ok(out)
}

#[cfg(test)]
#[path = "encrypt_tests.rs"]
mod tests;
