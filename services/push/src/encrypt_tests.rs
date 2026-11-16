use super::*;
use crate::b64;

/// The browser's side: read the header, agree on the secret with its own key, open the record.
fn decrypt(
    ua_private: EphemeralPrivateKey,
    ua_public: &[u8],
    auth: &[u8],
    body: &[u8],
) -> anyhow::Result<Vec<u8>> {
    let (salt, rest) = body.split_at(16);
    let (size, rest) = rest.split_at(4);
    assert_eq!(u32::from_be_bytes(size.try_into()?), RECORD_SIZE);
    let id_len = rest[0] as usize;
    let (as_public, sealed) = rest[1..].split_at(id_len);
    let shared = agree_ephemeral(
        ua_private,
        &UnparsedPublicKey::new(&ECDH_P256, as_public),
        <[u8]>::to_vec,
    )
    .map_err(|_| anyhow!("agree"))?;
    let (cek, nonce) = content_keys(&shared, auth, ua_public, as_public, salt)?;
    let key = LessSafeKey::new(UnboundKey::new(&AES_128_GCM, &cek).map_err(|_| anyhow!("key"))?);
    let mut record = sealed.to_vec();
    let open = key
        .open_in_place(
            Nonce::assume_unique_for_key(nonce),
            Aad::empty(),
            &mut record,
        )
        .map_err(|_| anyhow!("open"))?;
    assert_eq!(open.last(), Some(&2), "the last-record delimiter");
    Ok(open[..open.len() - 1].to_vec())
}

#[test]
fn only_the_subscribed_browser_reads_the_payload() -> anyhow::Result<()> {
    let rng = SystemRandom::new();
    let ua_private = EphemeralPrivateKey::generate(&ECDH_P256, &rng).map_err(|_| anyhow!("key"))?;
    let ua_public = ua_private
        .compute_public_key()
        .map_err(|_| anyhow!("public"))?;
    let mut auth = [0u8; 16];
    rng.fill(&mut auth).map_err(|_| anyhow!("rand"))?;
    let keys = Keys {
        p256dh: b64(ua_public.as_ref()),
        auth: b64(&auth),
    };
    let text = br#"{"title":"Replied to your project","link":"/projects/x?tab=chat"}"#;
    let body = encrypt(&keys, text)?;
    assert!(
        !body.windows(7).any(|w| w == b"Replied"),
        "the push service sees no text"
    );
    assert_eq!(decrypt(ua_private, ua_public.as_ref(), &auth, &body)?, text);
    Ok(())
}

#[test]
fn bad_keys_and_long_payloads_are_refused() {
    let keys = Keys {
        p256dh: b64(&[4u8; 10]),
        auth: b64(&[0u8; 16]),
    };
    assert!(encrypt(&keys, b"x").is_err());
    let rng = SystemRandom::new();
    let ua = EphemeralPrivateKey::generate(&ECDH_P256, &rng).map(|k| k.compute_public_key());
    let Ok(Ok(public)) = ua else { panic!("key") };
    let keys = Keys {
        p256dh: b64(public.as_ref()),
        auth: b64(&[0u8; 16]),
    };
    assert!(encrypt(&keys, &[b'a'; 4000]).is_err());
}
