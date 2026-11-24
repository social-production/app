import { t } from '#lib/i18n/index.ts';
import type { HomeFeedRow } from './homeFeed.ts';
import type { CharterMode, FeedSort, FeedWindow, ProjectCard } from './projects.ts';

/** The feed filter's Type pills (Scope Filters mockups §1), for Home, scopes and profiles. */
export type FeedType = 'all' | 'projects' | 'events' | 'requests' | 'threads' | 'posts';
export type FeedKind = Exclude<FeedType, 'all'>;
/** The kinds a feed shows; empty is All (owner, 2026-11-24: "should be able to select
 * multiple types, with all obviously only all"). */
export type FeedTypes = FeedKind[];

const KINDS: FeedKind[] = ['projects', 'events', 'requests', 'threads', 'posts'];

/** A tap on a Type pill: All clears the rest; a kind toggles, and picking every kind is All. */
export function toggleFeedType(types: FeedTypes, value: FeedType): FeedTypes {
  if (value === 'all') return [];
  const next = types.includes(value) ? types.filter((t) => t !== value) : [...types, value];
  return next.length === KINDS.length ? [] : KINDS.filter((k) => next.includes(k));
}

/** A saved Type, from before (one string) or now (a list). */
export function feedTypesFrom(saved: unknown): FeedTypes | null {
  const list = Array.isArray(saved) ? saved : typeof saved === 'string' ? [saved] : null;
  if (!list) return null;
  return KINDS.filter((k) => list.includes(k));
}

/** Every kind a feed holds has its own Type (owner, 2026-11-17: "type doesn't have all content
 * types, like request or post"). */
export const FEED_TYPES: { value: FeedType; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'projects', label: 'Projects' },
  { value: 'events', label: 'Events' },
  { value: 'requests', label: 'Requests' },
  { value: 'threads', label: 'Threads' },
  { value: 'posts', label: 'Posts' },
];

export const FEED_SORTS: { value: FeedSort; label: string }[] = [
  { value: 'trending', label: 'Trending' },
  { value: 'recent', label: t('Most recent') },
];

export const FEED_WINDOWS: { value: FeedWindow; label: string }[] = [
  { value: 'today', label: 'Today' },
  { value: 'week', label: 'Week' },
  { value: 'month', label: 'Month' },
  { value: 'all', label: t('All time') },
];

/** Which Type a card's mode falls under: events, help requests, threads and posts each
 * their own, and everything else (projects, services, proposals) Projects. */
export function typeOfMode(mode: CharterMode | string): FeedKind {
  if (mode === 'event') return 'events';
  if (mode === 'help-request') return 'requests';
  if (mode === 'thread') return 'threads';
  if (mode === 'post') return 'posts';
  return 'projects';
}

export const keepCard = (card: ProjectCard, types: FeedTypes): boolean =>
  types.length === 0 || types.includes(typeOfMode(card.mode));

/** A Home row under the Types: a project's activity counts as Projects; comments only show in All. */
export function keepRow(row: HomeFeedRow, types: FeedTypes): boolean {
  if (types.length === 0) return true;
  if ('activity' in row) return types.includes('projects');
  if ('comment' in row) return false;
  return keepCard(row.card, types);
}

/** "Trending · All time", the filter bar's summary. */
export function feedSummary(sort: FeedSort, window: FeedWindow, types: FeedTypes): string {
  const parts = [
    FEED_SORTS.find((s) => s.value === sort)?.label ?? 'Trending',
    FEED_WINDOWS.find((w) => w.value === window)?.label ?? t('All time'),
  ];
  if (types.length)
    parts.unshift(
      types.map((type) => t(FEED_TYPES.find((f) => f.value === type)?.label ?? '')).join(', '),
    );
  return parts.join(' · ');
}
