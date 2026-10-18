# native — Tauri 2 shell (ADR 0017)

The desktop app (Linux, macOS, Windows now; Android/iOS targets later) is a full node:

1. `src/node.rs` starts a Holochain conductor **in-process** (`holochain` crate), with data and keystore
   in the OS app-data dir and a random per-device keystore passphrase (OS keychain in M7).
2. It runs the **same gateway** as the seed node (`services/gateway`, as a library) on a random
   localhost port, so the UI reaches its own node exactly as a browser reaches a seed node.
3. `src/main.rs` opens the window on `ui/build` and injects `window.__SP_LOCAL_GATEWAY__`.
   `ui/src/lib/holo/nodes.ts` tries that first, then the configured gateways.

The hApp is compiled into the binary (`include_bytes!`), so build order is: `npm run build -w ui`,
then `bash scripts/build-happ.sh`, then `cargo build -p native`.

Run against a local seed node: `SP_BOOTSTRAP_URL=http://127.0.0.1:4433 cargo run -p native`
(an `http://` URL turns on plaintext relay, for dev only).
