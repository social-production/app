//! Native app (ADR 0017): starts this device's own Holochain node, then opens the UI
//! pointed at it. The same UI build is used by browsers; only the injected
//! `__SP_LOCAL_GATEWAY__` differs.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod node;

use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let node = tauri::async_runtime::block_on(node::start(data_dir))?;
            let inject = format!("window.__SP_LOCAL_GATEWAY__ = {:?};", node.gateway_url);
            app.manage(node);
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Social Production")
                .inner_size(1100.0, 800.0)
                .initialization_script(&inject)
                .build()?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build the Social Production app")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                // Close the conductor's databases cleanly before the process ends.
                let node = app.state::<node::Node>();
                let _ = tauri::async_runtime::block_on(node.conductor.clone().shutdown());
            }
        });
}
