import { CellType, setSigningCredentials, type CellId } from '@holochain/client';

import { connect, connection, isGuest } from './connection.ts';
import { gatewayApi } from './gateway.ts';
import { guestCell } from './guest.ts';

/** Cell ids resolved (or being resolved) this page view, by network seed. Two reads of a
 * project not opened before arrive together (the Links tab reads its page and its links);
 * sharing one lookup keeps them from cloning it twice. */
const known = new Map<string, Promise<CellId>>();

/**
 * The person's cell for one project (each project is a clone of the project DNA, seeded
 * `<creator key>.<nonce>`). Opening a project for the first time clones it, which joins
 * its network (not its membership); in a browser the gateway then grants this device.
 */
export async function projectCell(networkSeed: string, title: string): Promise<CellId> {
  // A guest reads the project through the gateway's archivist instead (guest.ts).
  if (!(await connect()) && isGuest()) return guestCell(networkSeed);
  return cloneOf('project', networkSeed, title);
}

/**
 * The person's cell for one direct or group chat (a clone of the conversation DNA, seeded
 * `<creator key>.<direct|group>.<nonce>`, ADR 0043). Only the seed's holders can find it.
 */
export function conversationCell(networkSeed: string): Promise<CellId> {
  return cloneOf('conversation', networkSeed, 'chat');
}

/** The network seeds of every clone of `role` on this node: the projects this person has
 * opened here, or their chats. */
export async function clonedSeeds(role: CloneRole): Promise<string[]> {
  const { ws } = await connection();
  const cells = (await ws.appInfo())?.cell_info[role] ?? [];
  return cells.flatMap((c) =>
    c.type === CellType.Cloned ? [c.value.dna_modifiers.network_seed] : [],
  );
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
    const created = await retryWhenBusy(() =>
      ws.createCloneCell({
        role_name: role,
        modifiers: { network_seed: networkSeed },
        name: title.slice(0, 100),
      }),
    );
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

/**
 * Creating a cell writes the conductor's own state database. When another write lands at the
 * same moment (the seed node's archivist joining a new project), SQLite refuses with "database
 * is locked" (517) and Holochain doesn't retry, so we do, briefly.
 */
async function retryWhenBusy<T>(write: () => Promise<T>): Promise<T> {
  for (let attempt = 1; ; attempt++) {
    try {
      return await write();
    } catch (e) {
      if (attempt >= 5 || !String(e).includes('database is locked')) throw e;
      await new Promise((r) => setTimeout(r, 400 * attempt));
    }
  }
}

function nonce(bytes: number): string {
  const random = crypto.getRandomValues(new Uint8Array(bytes));
  return btoa(String.fromCharCode(...random)).replace(/[+/=]/g, '');
}

/** A fresh network seed naming `creator` (see project_integrity::creator_in_seed). */
export function newProjectSeed(creator: string): string {
  return `${creator}.${nonce(9)}`;
}

/** A fresh private event's seed (sp_event::Audience, ADR 0061): its address is secret, so
 * the random part is as long as a chat's. */
export function newPrivateEventSeed(creator: string, organized: boolean): string {
  return `${creator}.${organized ? 'organized' : 'private'}.${nonce(24)}`;
}

/** A fresh chat seed (sp_core::conversation::chat_seed): whoever knows it can find the chat,
 * so the random part is long. */
export function newChatSeed(creator: string, kind: 'direct' | 'group'): string {
  return `${creator}.${kind}.${nonce(24)}`;
}
