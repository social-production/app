import type { ProjectCard } from './projects.ts';
import { isActivity, type RegionActivity, type RegionItem } from './regionFeed.ts';

/**
 * web's Home feed (utils/homeFeed.ts `mergeHomeFeed`): public rows and personal rows (the
 * people you follow) in one list. A personal item already in the public rows is dropped; "Most
 * recent" sorts by time, otherwise the two lists alternate.
 */
export type FeedScope = 'home' | 'global' | 'region';
export type HomeAudience = 'following' | 'popular';

export type HomeFeedRow =
  | { key: string; source: 'public' | 'personal'; card: ProjectCard }
  /** The Region feed's project-activity items (ADR 0084). */
  | { key: string; source: 'public'; activity: RegionActivity };

/** web: public rows by last activity, personal rows by creation. */
function rowTime(row: HomeFeedRow): number {
  if ('activity' in row) return row.activity.lastActivityAt;
  return row.source === 'public'
    ? row.card.lastActivityAt || row.card.createdAt
    : row.card.createdAt;
}

/** Public rows from feed cards, or from the Region feed's items. */
export function publicRows(items: (ProjectCard | RegionItem)[]): HomeFeedRow[] {
  return items.map((item) =>
    'kind' in item && isActivity(item)
      ? { key: `public:activity:${item.id}`, source: 'public', activity: item }
      : { key: `public:project:${item.id}`, source: 'public', card: item },
  );
}

export function mergeHomeFeed(
  publicItems: (ProjectCard | RegionItem)[],
  personalCards: ProjectCard[],
  sort: 'trending' | 'recent',
): HomeFeedRow[] {
  const shown = publicRows(publicItems);
  const seen = new Set(publicItems.map((item) => item.id));
  const personalRows: HomeFeedRow[] = personalCards
    .filter((card) => !seen.has(card.id))
    .map((card) => ({ key: `personal:project:${card.id}`, source: 'personal', card }));
  if (sort === 'recent') {
    return [...shown, ...personalRows].sort((a, b) => rowTime(b) - rowTime(a));
  }
  const mixed: HomeFeedRow[] = [];
  for (let i = 0; i < Math.max(shown.length, personalRows.length); i++) {
    const pub = shown[i];
    const own = personalRows[i];
    if (pub) mixed.push(pub);
    if (own) mixed.push(own);
  }
  return mixed;
}
