import { CellType, setSigningCredentials, type CellId } from '@holochain/client';

import { connection } from './connection.ts';
import { gatewayApi } from './gateway.ts';

/** Cell ids resolved (or being resolved) this page view, by network seed. Two reads of a
 * project not opened before arrive together (the Links tab reads its page and its links);
 * sharing one lookup keeps them from cloning it twice. */
const known = new Map<string, Promise<CellId>>();

/**
 * The person's cell for one project (each project is a clone of the project DNA, seeded
 * `<creator key>.<nonce>`). Opening a project for the first time clones it, which joins
 * its network (not its membership); in a browser the gateway then grants this device.
 */
export function projectCell(networkSeed: string, title: string): Promise<CellId> {
  const cached = known.get(networkSeed);
  if (cached) return cached;
  const resolving = resolveCell(networkSeed, title);
  known.set(networkSeed, resolving);
  resolving.catch(() => known.delete(networkSeed));
  return resolving;
}

async function resolveCell(networkSeed: string, title: string): Promise<CellId> {
  const { ws, session } = await connection();
  const existing = (await ws.appInfo())?.cell_info['project']?.find(
    (c) => c.type === CellType.Cloned && c.value.dna_modifiers.network_seed === networkSeed,
  );
  let cellId: CellId;
  if (existing?.type === CellType.Cloned) {
    cellId = existing.value.cell_id;
  } else {
    const created = await ws.createCloneCell({
      role_name: 'project',
      modifiers: { network_seed: networkSeed },
      name: title.slice(0, 100),
    });
    cellId = created.cell_id;
    if (session) {
      await gatewayApi.cells(session.gateway, {
        signingKey: Array.from(session.signingKey),
        deviceSecret: Array.from(session.deviceSecret),
        capSecret: Array.from(session.capSecret),
        dnaHashes: [Array.from(cellId[0])],
      });
    }
  }
  if (session) setSigningCredentials(cellId, session);
  return cellId;
}

/** A fresh network seed naming `creator` (see project_integrity::creator_in_seed). */
export function newProjectSeed(creator: string): string {
  const nonce = crypto.getRandomValues(new Uint8Array(9));
  return `${creator}.${btoa(String.fromCharCode(...nonce)).replace(/[+/=]/g, '')}`;
}
