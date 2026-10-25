import { decode, encode } from '@msgpack/msgpack';
import type { CellId } from '@holochain/client';

import { GatewayError } from './gateway.ts';
import { homeGateway } from './nodes.ts';

/**
 * Reads for signed-out people (web shows project pages to guests). The home gateway answers
 * a short list of read-only calls as its archivist (services/gateway/src/node/public.rs);
 * anything else is refused, so a guest's write fails as "Sign in to continue."
 */
export async function guestRead<T>(
  seed: string | undefined,
  zome: string,
  fn: string,
  payload?: unknown,
): Promise<T> {
  const gateway = homeGateway();
  if (!gateway) throw new Error('No node configured (VITE_SP_GATEWAYS).');
  const response = await fetch(new URL('/public/read', gateway), {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ seed, zome, fn, payload: Array.from(encode(payload ?? null)) }),
  });
  if (!response.ok) {
    const json: unknown = await response.json().catch(() => ({}));
    const message = (json as { error?: string }).error ?? `The server answered ${response.status}.`;
    throw new GatewayError(message, response.status);
  }
  return decode(new Uint8Array(await response.arrayBuffer())) as T;
}

/** Stand-in cell ids for projects a guest opens, by the seed the gateway reads them with. */
const guestCells = new WeakMap<CellId, string>();

export function guestCell(networkSeed: string): CellId {
  const cell: CellId = [new Uint8Array(39), new Uint8Array(39)];
  guestCells.set(cell, networkSeed);
  return cell;
}

export function guestSeedOf(cell: CellId): string | undefined {
  return guestCells.get(cell);
}
