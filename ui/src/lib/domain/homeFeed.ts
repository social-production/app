import type { CommentActivity } from './commentNotes.ts';
import type { Avatar } from './people.ts';
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
  | { key: string; source: 'public'; activity: RegionActivity }
  /** A comment by someone you follow (ADR 0107). */
  | { key: string; source: 'personal'; comment: CommentActivity };

/** web: public rows by last activity, personal rows by creation. */
function rowTime(row: HomeFeedRow): number {
  if ('activity' in row) return row.activity.lastActivityAt;
  if ('comment' in row) return row.comment.createdAt;
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
  comments: CommentActivity[] = [],
): HomeFeedRow[] {
  const shown = publicRows(publicItems);
  const seen = new Set(publicItems.map((item) => item.id));
  const cards: HomeFeedRow[] = personalCards
    .filter((card) => !seen.has(card.id))
    .map((card) => ({ key: `personal:project:${card.id}`, source: 'personal', card }));
  const said: HomeFeedRow[] = comments.map((comment) => ({
    key: `personal:comment:${comment.id}`,
    source: 'personal',
    comment,
  }));
  // Personal cards and comments interleave by time before they meet the public rows.
  const personalRows = [...cards, ...said].sort((a, b) => rowTime(b) - rowTime(a));
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

/** Home's line over a card made by someone you follow (Profiles and Comments mockups §1). */
export function madeContext(name: string, mode: string): string {
  if (mode === 'thread') return `${name} started this thread`;
  if (mode === 'help-request') return `${name} asked for help`;
  if (mode === 'event') return `${name} proposed this event`;
  return `${name} proposed this`;
}

/** The line over a followed person's card: what they did, with their picture. */
export interface FeedContext {
  text: string;
  name: string;
  avatar: Avatar | null;
}
