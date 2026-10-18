# 0021. Use Holochain's official runtime-tauri for the native app (supersedes 0017)

Status: accepted (2026-10-18)

## Context
ADR 0017 chose our own Tauri shell because the only known plugin (darksoil's tauri-plugin-holochain)
had no open licence and stopped at Holochain 0.6. Holochain has since published
[`runtime-tauri`](https://github.com/holochain/runtime-tauri): `tauri-plugin-hc` +
`holochain-conductor-runtime` 0.1.0 (MIT OR Apache-2.0, Holochain 0.7.1). Desktop and Android
are supported, and iOS is proven on a real device (wasmi interpreter, chosen by target).

## Decision
`native/` uses `tauri-plugin-hc`. The plugin boots the conductor in-process and serves the App API
plus zome-call signing to the webview over Tauri IPC. The UI detects `__HC_TAURI_HOLOCHAIN__` and
calls `AppWebsocket.connect()` with no session (`ui/src/lib/holo/connection.ts`). Browsers keep
the gateway path (`/session` + `/ws`). The app no longer embeds the gateway.

## Consequences
- Mobile packaging, signing and iOS quirks are maintained by Holochain, not us.
- No websocket or admin port is open on the device.
- The holo layer has two transports (IPC in the app, gateway in browsers), both inside `ui/src/lib/holo/`.
- iOS still needs Holochain's in-process keystore patch (unreleased; see ADR 0020) to be released
  or carried as a `[patch]` until then.
- The plugin's seed import/export is what M1's recovery phrase builds on.
