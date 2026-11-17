import type { HomeFeedRow } from './homeFeed.ts';
import type { CharterMode, FeedSort, FeedWindow, ProjectCard } from './projects.ts';

/** The feed filter's Type (Scope Filters mockups §1): one choice for Home, scopes and profiles. */
export type FeedType = 'all' | 'projects' | 'events' | 'threads';

export const FEED_TYPES: { value: FeedType; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'projects', label: 'Projects' },
  { value: 'events', label: 'Events' },
  { value: 'threads', label: 'Threads' },
];

export const FEED_SORTS: { value: FeedSort; label: string }[] = [
  { value: 'trending', label: 'Trending' },
  { value: 'recent', label: 'Most recent' },
];

export const FEED_WINDOWS: { value: FeedWindow; label: string }[] = [
  { value: 'today', label: 'Today' },
  { value: 'week', label: 'Week' },
  { value: 'month', label: 'Month' },
  { value: 'all', label: 'All time' },
];

/** Which Type a card's mode falls under: threads and posts are Threads, events Events, and
 * everything else (projects, services, proposals, help requests) Projects. */
export function typeOfMode(mode: CharterMode | string): Exclude<FeedType, 'all'> {
  if (mode === 'event') return 'events';
  if (mode === 'thread' || mode === 'post') return 'threads';
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
    FEED_WINDOWS.find((w) => w.value === window)?.label ?? 'All time',
  ];
  if (type !== 'all') parts.unshift(FEED_TYPES.find((t) => t.value === type)?.label ?? '');
  return parts.join(' · ');
}
