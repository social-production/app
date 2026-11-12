/**
 * The bootstrap/relay servers of this device's own node, in the native app (ADR 0091). The
 * app tries the saved server, then the shipped list, and boots with the first that answers;
 * these read that choice and save or forget a server (both restart the app).
 */

import type { AppWebsocket } from '@holochain/client';

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

/** "Help the network" (ADR 0094): this desktop runs a bootstrap server others can reach. */
export interface HelpSettings {
  enabled: boolean;
  port: number;
  /** How other people reach it: `http(s)://host[:port]`. */
  publicUrl: string;
}

export interface HelpState {
  settings: HelpSettings;
  running: boolean;
  error: string | null;
  /** What this desktop announces in the directory while its server runs. */
  announce: NetworkServer | null;
}

/** Whether this desktop helps, or null outside the app. */
export function helpNetworkState(): Promise<HelpState | null> {
  return invoke<HelpState | null>('help_network_state').catch(() => null);
}

/** Save the choice and restart the app to apply it. */
export async function setHelpNetwork(settings: HelpSettings): Promise<void> {
  await invoke<null>('set_help_network', { settings });
}

interface Announced {
  note: string;
}

/**
 * Once the device's node is connected: announce this desktop's server if it helps, and save
 * the servers other desktops announced, which the next start tries after Railway's. Best
 * effort; an older app's directory coordinator has no `list_servers`.
 */
export async function shareServers(ws: AppWebsocket): Promise<void> {
  const call = <T>(fn_name: string, payload: unknown = null): Promise<T> =>
    ws.callZome<T>({ role_name: 'directory', zome_name: 'health', fn_name, payload });
  const help = await helpNetworkState();
  if (help?.announce) await call('announce_server', JSON.stringify(help.announce));
  const heard = await call<Announced[]>('list_servers');
  await invoke<number>('remember_servers', { notes: heard.map((a) => a.note) });
}
