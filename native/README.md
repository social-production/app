# native — desktop + mobile app (ADR 0021)

Built on Holochain's official [runtime-tauri](https://github.com/holochain/runtime-tauri)
(`tauri-plugin-hc`). The device runs its own node:

1. `src/lib.rs` registers the plugin, which boots the conductor in-process with data in the OS
   app-data dir. The keystore passphrase is random per device (`src/keystore.rs`; OS keychain in M7).
2. When the conductor is ready, the bundled hApp (`include_bytes!`) is installed for this device's
   agent on first run, and the main window opens on `ui/build`.
3. The plugin injects `__HC_TAURI_HOLOCHAIN__`. The UI connects with `AppWebsocket.connect()` over
   Tauri IPC and the plugin signs zome calls. No websocket or admin port is opened.

Network: the Railway bootstrap/relay by default (`src/network.rs`). Override with
`SP_BOOTSTRAP_URL` (an `http://` URL enables plaintext relay, for local dev only).

Build order: `npm run build -w ui`, then `bash scripts/build-happ.sh`, then `cargo run -p native`.
Mobile (M7): the same crate builds as a library for Android and iOS (`tauri android|ios init`).
