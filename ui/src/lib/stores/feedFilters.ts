import { feedTypesFrom, type FeedTypes } from '#lib/domain/feedTypes.ts';
import type { FeedSort, FeedWindow } from '#lib/domain/projects.ts';

/**
 * A feed's filters kept on this device (owner, 2026-11-17: "if I set filters on feeds and
 * refresh it removes them"): one set for Home, one for channels, communities and the
 * Platform, one for profiles. Unknown or missing values fall back to the feed's defaults.
 */
export type FeedSurface = 'home' | 'scope' | 'profile';
export interface FeedFilters {
  type: FeedTypes;
  sort: FeedSort;
  window: FeedWindow;
}

const SORTS: readonly string[] = ['trending', 'recent', 'top', 'oldest'];
const WINDOWS: readonly string[] = ['today', 'week', 'month', 'all'];

export function readFeedFilters(surface: FeedSurface, fallback: FeedFilters): FeedFilters {
  try {
    const saved = JSON.parse(localStorage.getItem(`sp_feed_filters_${surface}`) ?? '{}') as {
      type?: unknown;
      sort?: string;
      window?: string;
    };
    const type = feedTypesFrom(saved.type);
    return {
      type: type ?? fallback.type,
      sort: SORTS.includes(saved.sort ?? '') ? (saved.sort as FeedSort) : fallback.sort,
      window: WINDOWS.includes(saved.window ?? '') ? (saved.window as FeedWindow) : fallback.window,
    };
  } catch {
    return fallback;
  }
}

export function keepFeedFilters(surface: FeedSurface, filters: FeedFilters): void {
  try {
    localStorage.setItem(`sp_feed_filters_${surface}`, JSON.stringify(filters));
  } catch {
    // Private windows: the filters last for this page only.
  }
}
