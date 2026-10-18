/**
 * Which node this client talks to (ADR 0013). Inside the native app it is the device's
 * own conductor over Tauri IPC (ADR 0021). In a browser it is the seed node's gateway:
 * a hosted person's account lives on the gateway they signed up with; moving to another
 * node is a recovery-phrase restore (docs/IDENTITY.md).
 */

/** Comma-separated gateway URLs baked in at build time, the person's home node first. */
const configured: string[] = String(import.meta.env.VITE_SP_GATEWAYS ?? '')
  .split(',')
  .map((url) => url.trim())
  .filter(Boolean);

/** True inside the native app, where Holochain's Tauri runtime injects its connection. */
export function inNativeApp(): boolean {
  return window.__HC_TAURI_HOLOCHAIN__ !== undefined;
}

export function homeGateway(): string | undefined {
  return configured[0];
}

export function describeGateway(url: string): string {
  return new URL(url).host;
}
