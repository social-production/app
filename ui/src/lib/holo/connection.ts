import { AppWebsocket } from '@holochain/client';

import { describe, inNativeApp, knownEndpoints, openSession, type NodeEndpoint } from './nodes.ts';

export type Role = 'directory' | 'project' | 'conversation';

export type ConnectionState =
  | { status: 'connecting' }
  | { status: 'connected'; node: string }
  | { status: 'offline'; reason: string };

type Listener = (state: ConnectionState) => void;

let client: AppWebsocket | undefined;
let pending: Promise<AppWebsocket> | undefined;
const listeners = new Set<Listener>();

export function onConnectionChange(listener: Listener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function emit(state: ConnectionState): void {
  for (const listener of listeners) listener(state);
}

async function connectTo(endpoint: NodeEndpoint): Promise<AppWebsocket> {
  const session = await openSession(endpoint);
  return AppWebsocket.connect({ url: new URL(session.appWsUrl), token: session.token });
}

/** The device's own node if we are the native app, else every known gateway in order
 *  until one accepts us (ADR 0013 failover). */
async function connectAny(): Promise<AppWebsocket> {
  emit({ status: 'connecting' });
  if (inNativeApp()) {
    // Holochain's Tauri runtime injected the connection and signs our zome calls.
    const ws = await AppWebsocket.connect();
    emit({ status: 'connected', node: 'this device' });
    return ws;
  }
  const errors: string[] = [];
  for (const endpoint of knownEndpoints()) {
    try {
      const ws = await connectTo(endpoint);
      emit({ status: 'connected', node: describe(endpoint) });
      return ws;
    } catch (error) {
      errors.push(`${describe(endpoint)}: ${String(error)}`);
    }
  }
  const reason = errors.length ? errors.join('; ') : 'no nodes configured';
  emit({ status: 'offline', reason });
  throw new Error(reason);
}

async function getClient(): Promise<AppWebsocket> {
  if (client) return client;
  pending ??= connectAny().finally(() => (pending = undefined));
  client = await pending;
  return client;
}

/** The single entry point for zome calls. Services wrap this; nothing else calls it. */
export async function callZome<T>(
  role: Role,
  zome: string,
  fn: string,
  payload?: unknown,
): Promise<T> {
  const ws = await getClient();
  try {
    return await ws.callZome<T>({ role_name: role, zome_name: zome, fn_name: fn, payload });
  } catch (error) {
    client = undefined; // drop a dead socket; the next call fails over
    throw error;
  }
}
