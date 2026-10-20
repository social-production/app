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
  return cloneOf('project', networkSeed, title);
}

/**
 * The person's cell for one direct or group chat (a clone of the conversation DNA, seeded
 * `<creator key>.<direct|group>.<nonce>`, ADR 0043). Only the seed's holders can find it.
 */
export function conversationCell(networkSeed: string): Promise<CellId> {
  return cloneOf('conversation', networkSeed, 'chat');
}

function cloneOf(role: CloneRole, networkSeed: string, name: string): Promise<CellId> {
  const cached = known.get(networkSeed);
  if (cached) return cached;
  const resolving = resolveCell(role, networkSeed, name);
  known.set(networkSeed, resolving);
  resolving.catch(() => known.delete(networkSeed));
  return resolving;
}

type CloneRole = 'project' | 'conversation';

async function resolveCell(role: CloneRole, networkSeed: string, title: string): Promise<CellId> {
  const { ws, session } = await connection();
  const existing = (await ws.appInfo())?.cell_info[role]?.find(
    (c) => c.type === CellType.Cloned && c.value.dna_modifiers.network_seed === networkSeed,
  );
  let cellId: CellId;
  if (existing?.type === CellType.Cloned) {
    cellId = existing.value.cell_id;
  } else {
    const created = await ws.createCloneCell({
      role_name: role,
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

function nonce(bytes: number): string {
  const random = crypto.getRandomValues(new Uint8Array(bytes));
  return btoa(String.fromCharCode(...random)).replace(/[+/=]/g, '');
}

/** A fresh network seed naming `creator` (see project_integrity::creator_in_seed). */
export function newProjectSeed(creator: string): string {
  return `${creator}.${nonce(9)}`;
}

/** A fresh chat seed (sp_core::conversation::chat_seed): whoever knows it can find the chat,
 * so the random part is long. */
export function newChatSeed(creator: string, kind: 'direct' | 'group'): string {
  return `${creator}.${kind}.${nonce(24)}`;
}
