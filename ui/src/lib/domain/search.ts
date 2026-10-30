/** web's search result kinds (types/search.ts) that the app has (no threads, posts or
 * activities yet). */
export type SearchResultKind =
  'project' | 'event' | 'channel' | 'community' | 'profile' | 'help-request';

/** web's `SearchEntityType` filter values for the kinds the app has. */
export type SearchType =
  'all' | 'project' | 'event' | 'help_request' | 'channel' | 'community' | 'user';

const SEARCH_TYPES: readonly string[] = [
  'project',
  'event',
  'help_request',
  'channel',
  'community',
  'user',
];

export interface SearchResultItem {
  id: string;
  kind: SearchResultKind;
  title: string;
  summary: string;
  meta: string;
  href: string;
}

/** web's searchKindLabels. */
export const searchKindLabels: Record<SearchResultKind, string> = {
  project: 'Project',
  event: 'Event',
  channel: 'Channel',
  community: 'Community',
  profile: 'Profile',
  'help-request': 'Help request',
};

/** web's DEFAULT_SUGGESTED_QUERIES: chips shown before anything is typed. */
export const suggestedQueries = ['platform', 'community', 'project', 'event'];

/** web's formatSearchMeta: schedule labels pass through; everything else shows its kind. */
export function formatSearchMeta(_meta: string, kind: SearchResultKind): string {
  return searchKindLabels[kind];
}

export function readSearchType(value: string | null): SearchType {
  return value && SEARCH_TYPES.includes(value) ? (value as SearchType) : 'all';
}

/** web-backend's `_scope_match_rank`: the handle itself, then a prefix, then anywhere. */
export function scopeMatchRank(name: string, slug: string, needle: string): number {
  const [n, s] = [name.toLowerCase(), slug.toLowerCase()];
  if (needle === n || needle === s) return 2;
  if (n.startsWith(needle) || s.startsWith(needle)) return 1;
  return 0.2;
}
