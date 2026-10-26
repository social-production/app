import type { ProjectCard } from './projects.ts';

/**
 * web's Home feed (utils/homeFeed.ts `mergeHomeFeed`): public rows and personal rows (the
 * people you follow) in one list. A personal item already in the public rows is dropped; "Most
 * recent" sorts by time, otherwise the two lists alternate.
 */
export type FeedScope = 'home' | 'global';
export type HomeAudience = 'following' | 'popular';

export interface HomeFeedRow {
  key: string;
  source: 'public' | 'personal';
  card: ProjectCard;
}

/** web: public rows by last activity, personal rows by creation. */
function rowTime(row: HomeFeedRow): number {
  return row.source === 'public'
    ? row.card.lastActivityAt || row.card.createdAt
    : row.card.createdAt;
}

export function mergeHomeFeed(
  publicCards: ProjectCard[],
  personalCards: ProjectCard[],
  sort: 'trending' | 'recent',
): HomeFeedRow[] {
  const publicRows: HomeFeedRow[] = publicCards.map((card) => ({
    key: `public:project:${card.id}`,
    source: 'public',
    card,
  }));
  const seen = new Set(publicCards.map((card) => card.id));
  const personalRows: HomeFeedRow[] = personalCards
    .filter((card) => !seen.has(card.id))
    .map((card) => ({ key: `personal:project:${card.id}`, source: 'personal', card }));
  if (sort === 'recent') {
    return [...publicRows, ...personalRows].sort((a, b) => rowTime(b) - rowTime(a));
  }
  const mixed: HomeFeedRow[] = [];
  for (let i = 0; i < Math.max(publicRows.length, personalRows.length); i++) {
    const pub = publicRows[i];
    const own = personalRows[i];
    if (pub) mixed.push(pub);
    if (own) mixed.push(own);
  }
  return mixed;
}
