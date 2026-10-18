//! Passkeys (`WebAuthn`) for hosted agents. Login is username-less: the browser offers the
//! person's passkey for this site and the credential id tells us whose agent it is.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context};
use webauthn_rs::prelude::*;

/// Pending ceremonies expire after this long.
const CEREMONY_TTL: Duration = Duration::from_mins(5);

type Pending<T> = Mutex<HashMap<Uuid, (T, Instant)>>;

pub struct Passkeys {
    webauthn: Webauthn,
    registrations: Pending<(String, PasskeyRegistration)>,
    logins: Pending<DiscoverableAuthentication>,
}

impl std::fmt::Debug for Passkeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Passkeys")
    }
}

impl Passkeys {
    pub fn new(rp_id: &str, rp_origin: &str) -> anyhow::Result<Self> {
        let origin = Url::parse(rp_origin).context("RP_ORIGIN must be a URL")?;
        let webauthn = WebauthnBuilder::new(rp_id, &origin)?
            .rp_name("Social Production")
            .build()?;
        Ok(Self {
            webauthn,
            registrations: Mutex::default(),
            logins: Mutex::default(),
        })
    }

    /// Start registering a passkey for `agent` (base64). Returns the ceremony id and the
    /// options for `navigator.credentials.create`, asking for a discoverable credential.
    pub fn start_registration(
        &self,
        agent: &str,
        name: &str,
    ) -> anyhow::Result<(Uuid, serde_json::Value)> {
        let user_id = Uuid::new_v4();
        let (options, state) = self
            .webauthn
            .start_passkey_registration(user_id, agent, name, None)?;
        let mut json = serde_json::to_value(options)?;
        let selection = &mut json["publicKey"]["authenticatorSelection"];
        selection["residentKey"] = "required".into();
        selection["requireResidentKey"] = true.into();
        // The device's own passkey store only: never prompt for a USB security key.
        selection["authenticatorAttachment"] = "platform".into();
        json["publicKey"]["hints"] = serde_json::json!(["client-device"]);
        let id = Uuid::new_v4();
        insert(&self.registrations, id, (agent.to_string(), state));
        Ok((id, json))
    }

    /// Finish registration: (agent, credential id, passkey json to store).
    pub fn finish_registration(
        &self,
        id: Uuid,
        credential: &RegisterPublicKeyCredential,
    ) -> anyhow::Result<(String, String, String)> {
        let (agent, state) = take(&self.registrations, id)?;
        let passkey = self
            .webauthn
            .finish_passkey_registration(credential, &state)?;
        let cred_id = credential_key(passkey.cred_id());
        Ok((agent, cred_id, serde_json::to_string(&passkey)?))
    }

    /// Options prefer this device's passkey, then one on the person's phone (QR), never
    /// a USB security key.
    pub fn start_login(&self) -> anyhow::Result<(Uuid, serde_json::Value)> {
        let (options, state) = self.webauthn.start_discoverable_authentication()?;
        let mut json = serde_json::to_value(options)?;
        json["publicKey"]["hints"] = serde_json::json!(["client-device", "hybrid"]);
        let id = Uuid::new_v4();
        insert(&self.logins, id, state);
        Ok((id, json))
    }

    /// The credential id the browser answered with, to look up the stored passkey.
    pub fn login_credential_id(&self, credential: &PublicKeyCredential) -> anyhow::Result<String> {
        let (_, cred_id) = self
            .webauthn
            .identify_discoverable_authentication(credential)?;
        Ok(credential_key(&cred_id.to_vec().into()))
    }

    /// Verify the login against the stored passkey; returns the updated passkey json.
    pub fn finish_login(
        &self,
        id: Uuid,
        credential: &PublicKeyCredential,
        stored: &str,
    ) -> anyhow::Result<String> {
        let state = take(&self.logins, id)?;
        let mut passkey: Passkey = serde_json::from_str(stored)?;
        let result = self.webauthn.finish_discoverable_authentication(
            credential,
            state,
            &[DiscoverableKey::from(&passkey)],
        )?;
        passkey.update_credential(&result);
        Ok(serde_json::to_string(&passkey)?)
    }
}

fn credential_key(id: &CredentialID) -> String {
    crate::hex::encode(id)
}

fn insert<T>(pending: &Pending<T>, id: Uuid, value: T) {
    let mut map = pending
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    map.retain(|_, (_, at)| at.elapsed() < CEREMONY_TTL);
    map.insert(id, (value, Instant::now()));
}

fn take<T>(pending: &Pending<T>, id: Uuid) -> anyhow::Result<T> {
    let mut map = pending
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    map.remove(&id)
        .filter(|(_, at)| at.elapsed() < CEREMONY_TTL)
        .map(|(value, _)| value)
        .ok_or_else(|| anyhow!("this sign-in step expired; please start again"))
}
