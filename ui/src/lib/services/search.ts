import type { CharterMode, ProjectCard } from '#lib/domain/projects.ts';
import type { Scope } from '#lib/domain/scopes.ts';
import {
  scopeMatchRank,
  type SearchResultItem,
  type SearchResultKind,
  type SearchType,
} from '#lib/domain/search.ts';
import { peopleIndex, projectFeed } from '#lib/holo/index.ts';

import { myClosedCommunities } from './closedCommunities.ts';
import { listScopes } from './scopes.ts';

const LIMIT = 20;
/** One kind of the project feed (events, help requests) is filtered from a wider page. */
const WIDE = 50;

interface PersonWire {
  agent: string;
  name: string;
  bio: string;
}

const ITEM_KIND: Partial<Record<CharterMode, SearchResultKind>> = {
  event: 'event',
  'help-request': 'help-request',
};

const ITEM_TYPE: Partial<Record<SearchType, SearchResultKind>> = {
  project: 'project',
  event: 'event',
  help_request: 'help-request',
};

const itemKind = (mode: CharterMode): SearchResultKind => ITEM_KIND[mode] ?? 'project';

const ITEM_PATH: Record<string, string> = {
  event: '/events',
  'help-request': '/help-requests',
  project: '/projects',
};

const wants = (type: SearchType, ...kinds: SearchType[]) => type === 'all' || kinds.includes(type);

/** web-backend's `_merge_scope_matches`: handles containing the query (two letters or more),
 * best match first; a closed community only for its members (`filter_search_results`). */
async function scopeMatches(q: string, type: SearchType): Promise<SearchResultItem[]> {
  if (q.length < 2 || !wants(type, 'channel', 'community')) return [];
  const [listed, closed] = await Promise.all([
    listScopes(q, WIDE),
    type === 'channel' ? [] : myClosedCommunities().catch((): Scope[] => []),
  ]);
  const needle = q.toLowerCase();
  const seen = new Set<string>();
  return [...closed, ...listed]
    .filter((s) => wants(type, s.kind) && `${s.handle} ${s.slug}`.toLowerCase().includes(needle))
    .filter((s) => !seen.has(`${s.kind}/${s.slug}`) && seen.add(`${s.kind}/${s.slug}`))
    .sort(
      (a, b) =>
        scopeMatchRank(b.handle, b.slug, needle) - scopeMatchRank(a.handle, a.slug, needle) ||
        a.handle.toLowerCase().localeCompare(b.handle.toLowerCase()),
    )
    .map((s) => ({
      id: s.id,
      kind: s.kind,
      title: s.handle,
      summary: s.description || s.handle,
      meta: s.kind,
      href: `/${s.kind === 'channel' ? 'channels' : 'communities'}/${s.slug}`,
    }));
}

async function itemMatches(q: string, type: SearchType): Promise<SearchResultItem[]> {
  const only = ITEM_TYPE[type];
  if (type !== 'all' && !only) return [];
  const page = await projectFeed<{ items: ProjectCard[] }>({
    q,
    sort: 'trending',
    window: 'all',
    offset: '0',
    limit: String(only ? WIDE : LIMIT),
  });
  return page.items
    .filter((c) => !only || itemKind(c.mode) === only)
    .map((c) => {
      const kind = itemKind(c.mode);
      return {
        id: c.id,
        kind,
        title: c.title,
        summary: c.summary || c.title,
        meta: kind,
        href: `${ITEM_PATH[kind]}/${c.id}`,
      };
    });
}

/**
 * web's search (web-backend `services/search.py`): channels and communities first, then
 * people (exact, prefix, then anywhere), then projects, events and help requests by
 * relevance. All come from the home node's indexes, so guests can search too.
 */
export async function search(query: string, type: SearchType): Promise<SearchResultItem[]> {
  const q = query.trim();
  if (!q) return [];
  const [scopes, people, items] = await Promise.all([
    scopeMatches(q, type),
    wants(type, 'user') ? peopleIndex<PersonWire[]>({ q, limit: String(LIMIT) }) : [],
    itemMatches(q, type),
  ]);
  const results: SearchResultItem[] = [
    ...scopes,
    ...people.map((p) => ({
      id: p.agent,
      kind: 'profile' as const,
      title: p.name,
      summary: p.bio || p.name,
      meta: 'user',
      href: `/profile/${p.agent}`,
    })),
    ...items,
  ];
  return results.slice(0, LIMIT);
}
