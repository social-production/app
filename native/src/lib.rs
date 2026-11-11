//! Native app (ADR 0021): this device runs its own Holochain node through Holochain's
//! official Tauri runtime. The UI connects over Tauri IPC (`__HC_TAURI_HOLOCHAIN__`),
//! so no websocket or admin port is opened. Same UI build as the browser PWA.

mod account;
mod keystore;
mod network;
mod places;
mod retire;
mod servers;

use tauri::{AppHandle, Listener, Manager};
use tauri_plugin_hc::{
    HolochainExt, HolochainPluginConfig, WindowOptions, EVENT_READY, EVENT_SETUP_FAILED,
};

const APP_ID: &str = "social-production";
/// The hApp is compiled in, so the app needs no download (build it first: scripts/build-happ.sh).
const HAPP: &[u8] = include_bytes!("../../happ/social_production.happ");

/// Start the app.
///
/// # Panics
/// If Tauri cannot start (no display, broken bundle): there is nothing to fall back to.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Warn)
                .level_for("native_lib", log::LevelFilter::Info)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            account::account_state,
            account::open_account,
            account::sign_out,
            places::locations,
            servers::network_servers,
            servers::reset_network_servers,
        ])
        .setup(|app| {
            app.handle().listen(EVENT_SETUP_FAILED, |event| {
                log::error!("holochain setup failed: {}", event.payload());
            });
            let handle = app.handle().clone();
            app.handle().listen(EVENT_READY, move |_| {
                network::log_stats(handle.clone());
                let handle = handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = Box::pin(open_main_window(handle)).await {
                        log::error!("failed to open main window: {e}");
                    }
                });
            });

            let data_dir = app.path().app_data_dir()?.join("holochain");
            std::fs::create_dir_all(&data_dir)?;
            let passphrase = keystore::passphrase(&data_dir)?;
            let server = servers::choose(&data_dir);
            app.manage(servers::managed_path(&data_dir));
            let config = HolochainPluginConfig::new(data_dir, network::config(&server));
            app.handle()
                .plugin(tauri_plugin_hc::init(passphrase, config))?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run the Social Production app");
}

/// Open the UI. The window starts unbound: the UI asks `account_state`, which binds it to
/// the person's app if this device has one, or shows sign-up / log-in (ADR 0090).
async fn open_main_window(handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let plugin = handle.holochain()?;
    let options = WindowOptions {
        title: Some("Social Production".into()),
        ..Default::default()
    };
    Box::pin(plugin.main_window_builder("main", None, options))
        .await?
        .build()?;
    Ok(())
}
