# native — desktop + mobile app (ADR 0021)

Built on Holochain's official [runtime-tauri](https://github.com/holochain/runtime-tauri)
(`tauri-plugin-hc`). The device runs its own node:

1. `src/lib.rs` registers the plugin, which boots the conductor in-process with data in the OS
   app-data dir. The keystore passphrase is random per device (`src/keystore.rs`; OS keychain in M7).
2. When the conductor is ready, the main window opens on `ui/build`, unbound. The UI asks
   `account_state` (`src/account.rs`): with no account it shows Sign up / Log in, and
   `open_account` installs the bundled hApp (`include_bytes!`) for the phrase's agent (ADR 0090).
3. The plugin injects `__HC_TAURI_HOLOCHAIN__`. The UI connects with `AppWebsocket.connect()` over
   Tauri IPC and the plugin signs zome calls. No websocket or admin port is opened.

Network (ADR 0091, `src/servers.rs`): the server saved in Settings → Network, then the shipped
list (Railway, then Holochain's public dev servers); the first whose `/health` answers at start
is used. `SP_BOOTSTRAP_URL` replaces the list (an `http://` URL enables plaintext relay, for
local dev only).

Build order: `npm run build -w ui`, then `bash scripts/build-happ.sh`, then `cargo run -p native`.
Mobile (M7): the same crate builds as a library for Android and iOS (`tauri android|ios init`).

Identity: the account is the 24-word recovery phrase, as on a gateway (ADR 0090): Sign up makes
new words, Log in brings an account back from the network. Pairing a second device with its own
key (ADR 0024) still works.

CI: `.github/workflows/desktop.yml` builds the app on Linux, Windows and macOS and runs
`tests/desktop/smoke.mjs` on Linux (tauri-driver + WebKitWebDriver under xvfb). Locally:
`cargo install tauri-driver --locked`, start `tauri-driver`, then
`node tests/desktop/smoke.mjs target/release/social-production`.
