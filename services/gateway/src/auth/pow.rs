//! Signup proof-of-work (docs/plans/M1-identity.md): the browser must find a nonce such
//! that sha256("<challenge>:<nonce>") starts with `bits` zero bits. Challenges are
//! stateless (timestamp + keyed hash) and single-use.

use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};

const MAX_AGE_SECS: u64 = 600;

#[derive(Debug)]
pub struct Pow {
    key: [u8; 32],
    bits: u32,
    used: Mutex<HashSet<String>>,
}

#[derive(Serialize)]
pub struct Challenge {
    pub challenge: String,
    pub bits: u32,
}

impl Pow {
    pub fn new(bits: u32) -> Self {
        let mut key = [0_u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self {
            key,
            bits,
            used: Mutex::new(HashSet::new()),
        }
    }

    pub fn issue(&self) -> Challenge {
        let issued = now();
        Challenge {
            challenge: format!("{issued}.{}", self.mac(issued)),
            bits: self.bits,
        }
    }

    /// True once per valid, fresh, solved challenge.
    pub fn verify(&self, challenge: &str, nonce: &str) -> bool {
        let Some((issued, mac)) = challenge.split_once('.') else {
            return false;
        };
        let Ok(issued) = issued.parse::<u64>() else {
            return false;
        };
        if mac != self.mac(issued) || now().saturating_sub(issued) > MAX_AGE_SECS {
            return false;
        }
        if leading_zero_bits(&Sha256::digest(format!("{challenge}:{nonce}"))) < self.bits {
            return false;
        }
        let mut used = self
            .used
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        used.retain(|c| {
            c.split_once('.')
                .and_then(|(t, _)| t.parse::<u64>().ok())
                .is_some_and(|t| now() - t <= MAX_AGE_SECS)
        });
        used.insert(challenge.to_string())
    }

    fn mac(&self, issued: u64) -> String {
        let digest = Sha256::new()
            .chain_update(self.key)
            .chain_update(issued.to_be_bytes())
            .finalize();
        crate::hex::encode(&digest[..16])
    }
}

fn leading_zero_bits(hash: &[u8]) -> u32 {
    let mut bits = 0;
    for byte in hash {
        if *byte == 0 {
            bits += 8;
        } else {
            return bits + byte.leading_zeros();
        }
    }
    bits
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::{leading_zero_bits, Pow};
    use sha2::{Digest, Sha256};

    #[test]
    fn solved_challenge_verifies_once() {
        let pow = Pow::new(8);
        let c = pow.issue();
        let nonce = (0..u64::MAX)
            .map(|n: u64| n.to_string())
            .find(|n| leading_zero_bits(&Sha256::digest(format!("{}:{n}", c.challenge))) >= 8)
            .expect("solvable");
        assert!(pow.verify(&c.challenge, &nonce));
        assert!(
            !pow.verify(&c.challenge, &nonce),
            "challenges are single-use"
        );
        assert!(!pow.verify("1.forged", &nonce));
    }
}
