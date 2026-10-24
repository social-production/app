import { CellType, decodeHashFromBase64, encodeHashToBase64, type CellId } from '@holochain/client';

import { connection, onDirectSignal } from './connection.ts';

/**
 * Direct signals through the inbox (the base conversation cell, ADR 0050): the conductor
 * passes them between people's nodes without running a zome or writing to the network,
 * which is what a call's ring, offer, answer and ICE candidates need.
 */

const GRANTED = 'sp.signals.granted.';

async function inboxCell(): Promise<CellId> {
  const { ws } = await connection();
  const cell = (await ws.appInfo())?.cell_info['conversation']?.find(
    (c) => c.type === CellType.Provisioned,
  );
  if (cell?.type !== CellType.Provisioned) throw new Error('No inbox cell.');
  return cell.value.cell_id;
}

/**
 * Let other people's nodes signal this person's inbox. The grant is a private entry on their
 * own chain; once per agent per device is enough (a repeat is harmless). Who may ring is
 * checked on arrival (services/calls.ts): only the other person in a direct chat.
 */
export async function acceptSignals(): Promise<void> {
  const cellId = await inboxCell();
  const key = GRANTED + encodeHashToBase64(cellId[1]);
  if (stored(key)) return;
  const { ws } = await connection();
  await ws.grantDirectSignalCapability({
    cell_id: cellId,
    tag: 'calls',
    constraint: { type: 'unrestricted' },
  });
  try {
    localStorage.setItem(key, '1');
  } catch {
    /* Private windows may refuse storage; the grant is simply repeated next time. */
  }
}

function stored(key: string): boolean {
  try {
    return localStorage.getItem(key) === '1';
  } catch {
    return false;
  }
}

/** Send `data` to one person's inbox. Best effort: nothing says whether it arrived. */
export async function sendSignal(to: string, data: Uint8Array): Promise<void> {
  const [dna] = await inboxCell();
  const { ws } = await connection();
  await ws.sendDirectSignal({ dna_hash: dna, agents: [decodeHashFromBase64(to)], signal: data });
}

/** Signals that reach this person's inbox, with the sender's key (vouched for by Holochain). */
export function onSignal(listener: (from: string, data: Uint8Array) => void): () => void {
  let inboxDna: string | null = null;
  void inboxCell().then(
    ([dna]) => (inboxDna = encodeHashToBase64(dna)),
    () => undefined,
  );
  return onDirectSignal((signal) => {
    if (signal.dna === inboxDna) listener(signal.from, signal.data);
  });
}
