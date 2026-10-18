import {
  decodeHashFromBase64,
  generateSigningKeyPair,
  setSigningCredentials,
  type CellId,
} from '@holochain/client';

/**
 * A seed/community node's gateway, which hands out app-websocket sessions (ADR 0013).
 * Inside the native app none of this is used: the device's own node is reached over
 * Tauri IPC instead (ADR 0021).
 */
export interface NodeEndpoint {
  url: string;
}

export interface NodeSession {
  appWsUrl: string;
  token: number[];
}

interface GatewaySession extends NodeSession {
  capSecret: number[];
  cells: { dna: string; agent: string }[];
}

/** Comma-separated gateway URLs baked in at build time, e.g. the Railway seed node first. */
const configured: string[] = String(import.meta.env.VITE_SP_GATEWAYS ?? '')
  .split(',')
  .map((url) => url.trim())
  .filter(Boolean);

/** True inside the native app, where Holochain's Tauri runtime injects its connection. */
export function inNativeApp(): boolean {
  return window.__HC_TAURI_HOLOCHAIN__ !== undefined;
}

/** Configured gateways, in preference order. */
export function knownEndpoints(): NodeEndpoint[] {
  return configured.map((url) => ({ url }));
}

/**
 * The browser signs zome calls with its own key; the gateway grants that key a
 * capability on our hosted cells (docs/IDENTITY.md#zome-call-signing).
 * M1: persist the key pair in IndexedDB instead of minting one per session.
 */
export async function openSession({ url }: NodeEndpoint): Promise<NodeSession> {
  const [keyPair, signingKey] = await generateSigningKeyPair();
  const response = await fetch(new URL('/session', url), {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ signingKey: Array.from(signingKey) }),
  });
  if (!response.ok) throw new Error(`gateway ${url}: ${response.status}`);
  const session = (await response.json()) as GatewaySession;

  const capSecret = new Uint8Array(session.capSecret);
  for (const cell of session.cells) {
    const cellId: CellId = [decodeHashFromBase64(cell.dna), decodeHashFromBase64(cell.agent)];
    setSigningCredentials(cellId, { capSecret, keyPair, signingKey });
  }
  return { appWsUrl: session.appWsUrl, token: session.token };
}

export function describe(endpoint: NodeEndpoint): string {
  return new URL(endpoint.url).host;
}
