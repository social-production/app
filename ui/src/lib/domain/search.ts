/** web's search result kinds (types/search.ts); the app has projects and people so far. */
export type SearchResultKind = 'project' | 'profile';

/** web's `SearchEntityType` filter values for the kinds the app has. */
export type SearchType = 'all' | 'project' | 'user';

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
  profile: 'Profile',
};

/** web's DEFAULT_SUGGESTED_QUERIES: chips shown before anything is typed. */
export const suggestedQueries = ['platform', 'community', 'project', 'event'];

/** web's formatSearchMeta: schedule labels pass through; everything else shows its kind. */
export function formatSearchMeta(_meta: string, kind: SearchResultKind): string {
  return searchKindLabels[kind];
}

export function readSearchType(value: string | null): SearchType {
  return value === 'project' || value === 'user' ? value : 'all';
}
