//! The person's account on this device (ADR 0090). The agent key is the recovery phrase's
//! seed, exactly as on a gateway (docs/IDENTITY.md), so the same 24 words give the same
//! person here, on a gateway and on any other device. The main window opens unbound; the UI
//! shows sign-up / log-in, and these commands install the hApp for that agent and bind it.

use std::time::Duration;

use holochain::conductor::api::AppInfo;
use holochain::prelude::{AgentPubKey, AppBundleSource, InstallAppPayload};
use holochain_types::app::AppStatus;
use tauri::{AppHandle, WebviewWindow};
use tauri_plugin_hc::{HolochainExt, Runtime};

use crate::{APP_ID, HAPP};

/// How long a log-in waits for the network to send the person's chain (ADR 0035's restore).
/// A desktop holds nothing yet, so it waits longer than a gateway's 45 s.
const RESTORE_WAIT: Duration = Duration::from_mins(2);
const POLL: Duration = Duration::from_millis(500);

/// Whether this device has an account; if it has, bind `window` to it (a reload re-runs
/// the page's start-up script unbound, so the UI asks on every start).
#[tauri::command]
pub async fn account_state(app: AppHandle, window: WebviewWindow) -> Result<bool, String> {
    let plugin = app.holochain().map_err(text)?;
    let runtime = plugin.try_runtime().map_err(text)?;
    if find(&runtime).await?.is_none() {
        return Ok(false);
    }
    Box::pin(bind(&app, &window)).await?;
    Ok(true)
}

/// Create (`restore: false`) or log in to (`restore: true`) the account whose 32-byte seed
/// is `seed`, then bind `window` to it. Returns the agent key (base64).
#[tauri::command]
pub async fn open_account(
    app: AppHandle,
    window: WebviewWindow,
    seed: Vec<u8>,
    restore: bool,
) -> Result<String, String> {
    let seed: [u8; 32] = seed
        .try_into()
        .map_err(|_| "A recovery phrase is 32 bytes.".to_string())?;
    let plugin = app.holochain().map_err(text)?;
    let runtime = plugin.try_runtime().map_err(text)?;
    if find(&runtime).await?.is_some() {
        return Err("This device already has an account. Sign out first.".into());
    }
    let agent = import(&runtime, seed).await?;
    install(&runtime, &agent, restore).await?;
    Box::pin(settle(&runtime, &agent)).await?;
    Box::pin(bind(&app, &window)).await?;
    Ok(holo_hash::AgentPubKeyB64::from(agent).to_string())
}

/// Remove the account from this device (its chain stays on the network; the words bring it
/// back). The window goes back to sign-up / log-in.
#[tauri::command]
pub async fn sign_out(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    let plugin = app.holochain().map_err(text)?;
    let runtime = plugin.try_runtime().map_err(text)?;
    Box::pin(plugin.rebind_window(window.label(), None))
        .await
        .map_err(text)?;
    if find(&runtime).await?.is_some() {
        runtime.uninstall_app(APP_ID.into()).await.map_err(text)?;
    }
    Ok(())
}

/// Import the seed into the keystore, or reuse it: lair never deletes an entry, so logging in
/// again after signing out finds the key already there (a UNIQUE constraint on its key).
async fn import(runtime: &Runtime, seed: [u8; 32]) -> Result<AgentPubKey, String> {
    match runtime.import_key_seed(seed).await {
        Ok(agent) => Ok(agent),
        Err(e) if e.to_string().contains("UNIQUE") => {
            let mut public = [0_u8; sodoken::sign::PUBLICKEYBYTES];
            let mut secret = sodoken::SizedLockedArray::<{ sodoken::sign::SECRETKEYBYTES }>::new()
                .map_err(text)?;
            sodoken::sign::seed_keypair(&mut public, &mut secret.lock(), &seed).map_err(text)?;
            Ok(AgentPubKey::from_raw_32(public.to_vec()))
        }
        Err(e) => Err(e.to_string()),
    }
}

async fn find(runtime: &Runtime) -> Result<Option<AppInfo>, String> {
    let apps = runtime.list_apps().await.map_err(text)?;
    Ok(apps.into_iter().find(|app| app.installed_app_id == APP_ID))
}

async fn install(runtime: &Runtime, agent: &AgentPubKey, restore: bool) -> Result<(), String> {
    let payload = InstallAppPayload {
        source: AppBundleSource::Bytes(HAPP.to_vec().into()),
        agent_key: Some(agent.clone()),
        installed_app_id: Some(APP_ID.into()),
        network_seed: None,
        roles_settings: None,
        ignore_genesis_failure: false,
        restore_from_dht: restore,
    };
    Box::pin(runtime.install_app(payload)).await.map_err(text)?;
    Ok(())
}

/// Bring the app to Enabled. A restore runs in the background until the chain arrives; a
/// phrase the network has no chain for (new, or from a reset network) starts fresh.
async fn settle(runtime: &Runtime, agent: &AgentPubKey) -> Result<(), String> {
    let started = tokio::time::Instant::now();
    loop {
        let app = find(runtime)
            .await?
            .ok_or_else(|| "The app vanished while installing.".to_string())?;
        match app.status {
            AppStatus::Enabled => return Ok(()),
            AppStatus::Disabled(_) => {
                runtime.enable_app(APP_ID.into()).await.map_err(text)?;
            }
            AppStatus::AwaitingRestore if started.elapsed() < RESTORE_WAIT => {
                tokio::time::sleep(POLL).await;
            }
            AppStatus::AwaitingRestore | AppStatus::Unrecoverable(..) => {
                log::info!("nothing to restore; starting a fresh chain");
                runtime.uninstall_app(APP_ID.into()).await.map_err(text)?;
                install(runtime, agent, false).await?;
            }
            AppStatus::AwaitingMemproofs => {
                return Err("The app is waiting for membrane proofs.".into());
            }
        }
    }
}

/// Always rebind, even when the window is already bound: after a reload the page's injected
/// app id is empty again, and only the rebind event sets it.
async fn bind(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let plugin = app.holochain().map_err(text)?;
    Box::pin(plugin.rebind_window(window.label(), Some(APP_ID.into())))
        .await
        .map_err(text)
}

#[allow(clippy::needless_pass_by_value)] // used as `map_err(text)`
fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}
