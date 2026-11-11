//! The statement that retires a gateway's hosted copy of this account (ADR 0092). After a
//! log-in brings the person's chain to this computer, the UI sends it to the gateways; each
//! checks the signature against the agent key and stops hosting the account, so one agent
//! no longer runs on two nodes at once. The seed never leaves the device.

use serde::Serialize;

/// What the UI posts to a gateway's `/account/retire`.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Retirement {
    pub agent: String,
    pub at: i64,
    pub signature: Vec<u8>,
}

/// The bytes a gateway verifies (`services/gateway/src/routes/retire.rs::statement`).
fn statement(agent: &str, at: i64) -> Vec<u8> {
    format!("social-production: retire the hosted copy of {agent} at {at}").into_bytes()
}

/// Sign the retirement of `agent` (the base64 key of `seed`) now.
pub fn sign(seed: &[u8; 32], agent: &str) -> Result<Retirement, String> {
    let at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let at = i64::try_from(at).map_err(|e| e.to_string())?;
    let mut public = [0_u8; sodoken::sign::PUBLICKEYBYTES];
    let mut secret = sodoken::SizedLockedArray::<{ sodoken::sign::SECRETKEYBYTES }>::new()
        .map_err(|e| e.to_string())?;
    sodoken::sign::seed_keypair(&mut public, &mut secret.lock(), seed)
        .map_err(|e| e.to_string())?;
    let mut signature = [0_u8; sodoken::sign::SIGNATUREBYTES];
    sodoken::sign::sign_detached(&mut signature, &statement(agent, at), &secret.lock())
        .map_err(|e| e.to_string())?;
    Ok(Retirement {
        agent: agent.to_string(),
        at,
        signature: signature.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::{sign, statement};

    #[test]
    fn the_signature_verifies_with_the_phrase_key() {
        let seed = [9_u8; 32];
        let mut public = [0_u8; sodoken::sign::PUBLICKEYBYTES];
        let mut secret = sodoken::SizedLockedArray::new().expect("locked memory");
        sodoken::sign::seed_keypair(&mut public, &mut secret.lock(), &seed).expect("keypair");
        let signed = sign(&seed, "uhCAkexample").expect("sign");
        let signature: [u8; 64] = signed.signature.clone().try_into().expect("64 bytes");
        let message = statement("uhCAkexample", signed.at);
        assert!(sodoken::sign::verify_detached(
            &signature, &message, &public
        ));
        // The gateway's `statement` must write exactly these bytes.
        assert_eq!(
            statement("A", 1),
            b"social-production: retire the hosted copy of A at 1"
        );
    }
}
