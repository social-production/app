import { t } from '#lib/i18n/index.ts';
import type { HomeFeedRow } from './homeFeed.ts';
import type { CharterMode, FeedSort, FeedWindow, ProjectCard } from './projects.ts';

/** The feed filter's Type (Scope Filters mockups §1): one choice for Home, scopes and profiles. */
export type FeedType = 'all' | 'projects' | 'events' | 'requests' | 'threads' | 'posts';

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
export function typeOfMode(mode: CharterMode | string): Exclude<FeedType, 'all'> {
  if (mode === 'event') return 'events';
  if (mode === 'help-request') return 'requests';
  if (mode === 'thread') return 'threads';
  if (mode === 'post') return 'posts';
  return 'projects';
}

export const keepCard = (card: ProjectCard, type: FeedType): boolean =>
  type === 'all' || typeOfMode(card.mode) === type;

/** A Home row under a Type: a project's activity counts as Projects; comments only show in All. */
export function keepRow(row: HomeFeedRow, type: FeedType): boolean {
  if (type === 'all') return true;
  if ('activity' in row) return type === 'projects';
  if ('comment' in row) return false;
  return keepCard(row.card, type);
}

/** "Trending · All time", the filter bar's summary. */
export function feedSummary(sort: FeedSort, window: FeedWindow, type: FeedType): string {
  const parts = [
    FEED_SORTS.find((s) => s.value === sort)?.label ?? 'Trending',
    FEED_WINDOWS.find((w) => w.value === window)?.label ?? t('All time'),
  ];
  if (type !== 'all') parts.unshift(FEED_TYPES.find((t) => t.value === type)?.label ?? '');
  return parts.join(' · ');
}
