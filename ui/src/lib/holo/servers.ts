/**
 * The bootstrap/relay servers of this device's own node, in the native app (ADR 0091). The
 * app tries the saved server, then the shipped list, and boots with the first that answers;
 * these read that choice and save or forget a server (both restart the app).
 */

export interface NetworkServer {
  bootstrap: string;
  relay: string;
}

export interface NetworkServers {
  current: NetworkServer;
  saved: NetworkServer | null;
  shipped: NetworkServer[];
  /** Set by `SP_BOOTSTRAP_URL` (local dev, CI): Settings can't change it. */
  fixed: boolean;
}

interface TauriWindow {
  __TAURI_INTERNALS__?: {
    invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  };
}

function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const internals = (window as TauriWindow).__TAURI_INTERNALS__;
  if (!internals) return Promise.reject(new Error('Not running in the app.'));
  return internals.invoke<T>(command, args);
}

/** The servers this device knows, or null outside the app. */
export function networkServers(): Promise<NetworkServers | null> {
  return invoke<NetworkServers | null>('network_servers').catch(() => null);
}

/** Save `url` (bootstrap and relay at one address, as on our servers) and restart the app. */
export async function saveNetworkServer(url: string): Promise<void> {
  const address = new URL(url.trim());
  if (address.protocol !== 'https:') throw new Error('The address must start with https://');
  const clean = address.origin;
  await invoke<null>('plugin:hc|set_user_network_config', {
    bootstrapUrl: clean,
    relayUrl: clean,
  });
}

/** Forget the saved server and restart on the shipped list. */
export async function resetNetworkServer(): Promise<void> {
  await invoke<null>('reset_network_servers');
}
