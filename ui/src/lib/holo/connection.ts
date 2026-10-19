import {
  AppWebsocket,
  CellType,
  decodeHashFromBase64,
  encodeHashToBase64,
  setSigningCredentials,
  type CellId,
} from '@holochain/client';

import { gatewayApi, GatewayError } from './gateway.ts';
import { forgetSession, loadSession, type DeviceSession } from './keys.ts';
import { describeGateway, homeGateway, inNativeApp } from './nodes.ts';

export type Role = 'directory' | 'project' | 'conversation';

export type ConnectionState =
  | { status: 'connecting' }
  | { status: 'signed-out' }
  | { status: 'connected'; node: string; agent: string }
  | { status: 'offline'; reason: string };

let client: AppWebsocket | undefined;
/** The browser's gateway session (absent in the native app, which signs for itself). */
let deviceSession: DeviceSession | undefined;
let pending: Promise<AppWebsocket | undefined> | undefined;
let state: ConnectionState = { status: 'connecting' };
const listeners = new Set<(state: ConnectionState) => void>();

export function onConnectionChange(listener: (state: ConnectionState) => void): () => void {
  listeners.add(listener);
  listener(state);
  return () => listeners.delete(listener);
}

function emit(next: ConnectionState): void {
  state = next;
  for (const listener of listeners) listener(state);
}

/** Connect using the device's own node (app) or its stored gateway session (browser). */
export function connect(): Promise<AppWebsocket | undefined> {
  if (client) return Promise.resolve(client);
  pending ??= open().finally(() => (pending = undefined));
  return pending;
}

async function open(): Promise<AppWebsocket | undefined> {
  emit({ status: 'connecting' });
  try {
    if (inNativeApp()) return await openNative();
    const gateway = homeGateway();
    if (!gateway) throw new Error('No node configured (VITE_SP_GATEWAYS).');
    const stored = await loadSession(gateway);
    if (!stored) return signedOut();
    const resumed = await gatewayApi.resume(gateway, Array.from(stored.signingKey));
    return await attach(stored, resumed.appWsUrl, resumed.token);
  } catch (error) {
    if (error instanceof GatewayError && error.status === 401) return signedOut();
    emit({ status: 'offline', reason: error instanceof Error ? error.message : String(error) });
    return undefined;
  }
}

async function openNative(): Promise<AppWebsocket> {
  // Holochain's Tauri runtime injected the connection and signs our zome calls.
  client = await AppWebsocket.connect();
  const agent = (await client.appInfo())?.agent_pub_key;
  emit({ status: 'connected', node: 'this device', agent: agent ? encodeHashToBase64(agent) : '' });
  return client;
}

/**
 * Use a session from sign-in (holo/account.ts) or from storage. `beforeShow` runs once
 * the socket is open but before the app is shown (e.g. writing a new person's profile).
 */
export async function attach(
  session: DeviceSession,
  appWsUrl: string,
  token: number[],
  beforeShow?: () => Promise<void>,
): Promise<AppWebsocket> {
  for (const cell of session.cells) {
    const cellId: CellId = [decodeHashFromBase64(cell.dna), decodeHashFromBase64(cell.agent)];
    setSigningCredentials(cellId, session);
  }
  client = await AppWebsocket.connect({ url: new URL(appWsUrl), token });
  deviceSession = session;
  // Project cells joined since sign-in were granted to this key too (/session/cells).
  for (const cells of Object.values((await client.appInfo())?.cell_info ?? {})) {
    for (const cell of cells) {
      if (cell.type === CellType.Cloned) setSigningCredentials(cell.value.cell_id, session);
    }
  }
  await beforeShow?.();
  emit({ status: 'connected', node: describeGateway(session.gateway), agent: session.agent });
  return client;
}

/** Forget this device's session. Reloading drops the open websocket with the page. */
export async function signOut(): Promise<void> {
  const gateway = homeGateway();
  if (gateway) await forgetSession(gateway);
  client = undefined;
  deviceSession = undefined;
  window.location.reload();
}

function signedOut(): undefined {
  emit({ status: 'signed-out' });
  return undefined;
}

/** The single entry point for zome calls. Services wrap this; nothing else calls it. */
export async function callZome<T>(
  role: Role,
  zome: string,
  fn: string,
  payload?: unknown,
): Promise<T> {
  const ws = await connect();
  if (!ws) throw new Error('Not signed in.');
  try {
    return await ws.callZome<T>({ role_name: role, zome_name: zome, fn_name: fn, payload });
  } catch (error) {
    if (String(error).toLowerCase().includes('closed')) client = undefined; // reconnect next call
    throw error;
  }
}

/** The open socket and, in a browser, the device session (for granting new cells). */
export async function connection(): Promise<{ ws: AppWebsocket; session?: DeviceSession }> {
  const ws = await connect();
  if (!ws) throw new Error('Not signed in.');
  return deviceSession ? { ws, session: deviceSession } : { ws };
}

/** A zome call on a specific cell (a project or conversation clone). */
export async function callCell<T>(
  cellId: CellId,
  zome: string,
  fn: string,
  payload?: unknown,
): Promise<T> {
  const { ws } = await connection();
  return ws.callZome<T>({ cell_id: cellId, zome_name: zome, fn_name: fn, payload });
}
