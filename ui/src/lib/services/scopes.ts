import type { ProjectCard } from '#lib/domain/projects.ts';
import {
  PLATFORM_DESCRIPTION,
  PLATFORM_SLUG,
  slugOf,
  type Scope,
  type ScopeKind,
} from '#lib/domain/scopes.ts';
import { scopeIndex } from '#lib/holo/index.ts';

import { createClosedCommunity, myClosedCommunities } from './closedCommunities.ts';
import { myAgent } from './people.ts';
import { createProject, myProjectSeeds } from './projects.ts';

/** Scopes created on this device, until the node's index lists them (it reads new spaces
 * within a minute or so). */
const justCreated = new Map<string, Scope>();

const toScope = (c: ProjectCard): Scope => ({
  id: c.id,
  kind: c.mode as ScopeKind,
  handle: c.title,
  slug: slugOf(c.title),
  description: c.summary,
  members: c.members,
  creator: c.creator,
  createdAt: c.createdAt,
  networkSeed: c.networkSeed,
});

/** Channels and communities whose handle or description matches, first handle first. */
export async function listScopes(q = '', limit = 50): Promise<Scope[]> {
  const cards = await scopeIndex<ProjectCard[]>({ q, limit: String(limit) });
  const listed = cards.map(toScope);
  // Until the index reads them, this device's new scopes still show (e.g. as tag choices).
  const wanted = q.trim().toLowerCase();
  const pending = [...justCreated.values()].filter(
    (s) =>
      !listed.some((l) => l.slug === s.slug) &&
      (s.handle.toLowerCase().includes(wanted) || s.description.toLowerCase().includes(wanted)),
  );
  return [...pending, ...listed].slice(0, limit);
}

/** The scope a handle names (web: handles are unique across channels and communities). A
 * closed community you're in comes first: nobody outside it can see its handle to avoid it. */
export async function findScope(slug: string, kind?: ScopeKind): Promise<Scope | null> {
  const wanted = slugOf(slug);
  if (kind !== 'channel') {
    const closed = (await myClosedCommunities().catch(() => [])).find((s) => s.slug === wanted);
    if (closed) return closed;
  }
  const found = (await listScopes(wanted)).filter(
    (s) => s.slug === wanted && (!kind || s.kind === kind),
  );
  const first = found.sort((a, b) => a.createdAt - b.createdAt)[0];
  if (first) return first;
  const mine = justCreated.get(wanted);
  return mine && (!kind || mine.kind === kind) ? mine : null;
}

/** web's createChannel / createCommunity: refuses a taken handle, then founds the space; a
 * private community (`closed`) gets its own network instead of a listing. */
export async function createScope(
  kind: ScopeKind,
  handle: string,
  description: string,
  closed?: { organized: boolean },
): Promise<string> {
  const slug = slugOf(handle);
  if (await findScope(slug))
    throw new Error('That handle is already used by a channel or community');
  if (kind === 'community' && closed) {
    return (await createClosedCommunity(handle, description, closed.organized)).slug;
  }
  await found(kind, handle.trim(), description.trim());
  return slug;
}

/** The Platform channel (ADR 0095): the earliest record, or a new one with web's charter
 * (anyone may write it; readers take the earliest). Its creator is its first follower. */
export async function foundPlatform(): Promise<Scope> {
  return (
    (await findScope(PLATFORM_SLUG, 'channel')) ??
    (await found('channel', PLATFORM_SLUG, PLATFORM_DESCRIPTION))
  );
}

async function found(kind: ScopeKind, handle: string, description: string): Promise<Scope> {
  const slug = slugOf(handle);
  const id = await createProject({
    title: handle,
    description,
    mode: kind,
    subtype: 'open',
    location_label: '',
    phase: '',
    tags: [],
  });
  const scope: Scope = {
    id,
    kind,
    handle,
    slug,
    description,
    members: 1,
    creator: await myAgent(),
    createdAt: Date.now() * 1000,
    networkSeed: '',
  };
  justCreated.set(slug, scope);
  return scope;
}

/** The scopes you joined: the index's scopes whose space is among yours, then your closed
 * communities. */
export async function myScopes(): Promise<Scope[]> {
  const [all, mine, closed] = await Promise.all([
    listScopes('', 200),
    myProjectSeeds(),
    myClosedCommunities().catch(() => []),
  ]);
  const seeds = new Set(mine);
  return [...all.filter((s) => seeds.has(s.networkSeed)), ...closed];
}
