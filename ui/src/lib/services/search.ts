import type { ProjectCard } from '#lib/domain/projects.ts';
import type { SearchResultItem, SearchType } from '#lib/domain/search.ts';
import { peopleIndex, projectFeed } from '#lib/holo/index.ts';

const LIMIT = 20;

interface PersonWire {
  agent: string;
  name: string;
  bio: string;
}

/**
 * web's search (web-backend `services/search.py`): people first (exact, prefix, then
 * anywhere), then projects by relevance. Both come from the home node's indexes, so guests
 * can search too.
 */
export async function search(query: string, type: SearchType): Promise<SearchResultItem[]> {
  const q = query.trim();
  if (!q) return [];
  const [people, projects] = await Promise.all([
    type === 'project' ? [] : peopleIndex<PersonWire[]>({ q, limit: String(LIMIT) }),
    type === 'user'
      ? { items: [] }
      : projectFeed<{ items: ProjectCard[] }>({
          q,
          sort: 'trending',
          window: 'all',
          offset: '0',
          limit: String(LIMIT),
        }),
  ]);
  const results: SearchResultItem[] = [
    ...people.map((p) => ({
      id: p.agent,
      kind: 'profile' as const,
      title: p.name,
      summary: p.bio || p.name,
      meta: 'user',
      href: `/profile/${p.agent}`,
    })),
    ...projects.items.map((c) => ({
      id: c.id,
      kind: 'project' as const,
      title: c.title,
      summary: c.summary || c.title,
      meta: 'project',
      href: `/projects/${c.id}`,
    })),
  ];
  return results.slice(0, LIMIT);
}
