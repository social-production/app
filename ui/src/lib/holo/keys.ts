import type { generateSigningKeyPair } from '@holochain/client';

/** The client's signing key pair (libsodium's shape, not re-exported by the client). */
export type KeyPair = Awaited<ReturnType<typeof generateSigningKeyPair>>[0];

/**
 * This device's session with a gateway: its own zome-call signing key pair (granted access
 * to the person's cells at sign-in) and the cap secret. Kept in IndexedDB so reloads don't
 * need a passkey prompt; signing out deletes it.
 */
export interface DeviceSession {
  gateway: string;
  agent: string;
  keyPair: KeyPair;
  signingKey: Uint8Array;
  capSecret: Uint8Array;
  /** Proves this device to the gateway (adding a passkey). */
  deviceSecret: Uint8Array;
  cells: { dna: string; agent: string }[];
}

const DB = 'social-production';
const STORE = 'device-sessions';

function open(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB, 1);
    request.onupgradeneeded = () => request.result.createObjectStore(STORE);
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error('IndexedDB unavailable'));
  });
}

async function run<T>(mode: IDBTransactionMode, op: (s: IDBObjectStore) => IDBRequest): Promise<T> {
  const db = await open();
  return new Promise((resolve, reject) => {
    const request = op(db.transaction(STORE, mode).objectStore(STORE));
    request.onsuccess = () => resolve(request.result as T);
    request.onerror = () => reject(request.error ?? new Error('IndexedDB request failed'));
  });
}

export function loadSession(gateway: string): Promise<DeviceSession | undefined> {
  return run('readonly', (s) => s.get(gateway));
}

export async function saveSession(session: DeviceSession): Promise<void> {
  await run('readwrite', (s) => s.put(session, session.gateway));
}

export async function forgetSession(gateway: string): Promise<void> {
  await run('readwrite', (s) => s.delete(gateway));
}
