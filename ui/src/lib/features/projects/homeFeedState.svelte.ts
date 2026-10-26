import {
  mergeHomeFeed,
  type FeedScope,
  type HomeAudience,
  type HomeFeedRow,
} from '#lib/domain/homeFeed.ts';
import type { FeedSort, FeedWindow, ProjectCard, Stance } from '#lib/domain/projects.ts';
import { followCircles } from '#lib/services/follows.ts';

import { FeedState } from './feedState.svelte.ts';

const KEYS = { scope: 'sp_feed_scope', audience: 'sp_home_audience' };

function read<T extends string>(key: string, allowed: readonly T[], fallback: T): T {
  try {
    const value = localStorage.getItem(key);
    return allowed.includes(value as T) ? (value as T) : fallback;
  } catch {
    return fallback;
  }
}

function keep(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {
    // Private windows may refuse storage; the choice then lasts for this page only.
  }
}

/**
 * web's PublicFeed with its default `combineFeeds` (the app has no setting for it): signed in,
 * the public rows (Global, or Home's joined channels and communities, none until M5d) merge
 * with the personal feed, which is the projects of people you follow. "Following + popular"
 * adds discovery posts and threads on web; the app has none yet. Scope and audience are kept
 * per device (web keeps them in its settings).
 */
export class HomeFeedState {
  scope = $state<FeedScope>(read(KEYS.scope, ['home', 'global'], 'global'));
  audience = $state<HomeAudience>(read(KEYS.audience, ['following', 'popular'], 'popular'));
  readonly public = new FeedState(() => this.search());
  readonly personal = new FeedState(() => '');
  private signedIn = $state(false);

  constructor(private readonly search: () => string) {}

  get sort(): FeedSort {
    return this.public.sort;
  }

  get window(): FeedWindow {
    return this.public.window;
  }

  /** What the stack shows: Home before the first load settles shows "Loading feed...". */
  /** Signed out, Home shows Global as on web (the choice is kept for when you sign in). */
  readonly shownScope = $derived<FeedScope>(this.signedIn ? this.scope : 'global');

  readonly rows = $derived.by((): HomeFeedRow[] => {
    const shown = this.shownScope === 'home' ? [] : this.public.items;
    if (!this.signedIn) return mergeHomeFeed(shown, [], 'trending');
    const sort = this.public.sort === 'recent' ? 'recent' : 'trending';
    return mergeHomeFeed(shown, this.personal.items, sort);
  });

  readonly loading = $derived(this.public.loading || this.personal.loading);
  readonly error = $derived(this.public.error ?? this.personal.error);
  readonly hasMore = $derived(
    (this.shownScope === 'global' && this.public.hasMore) ||
      (this.signedIn && this.personal.hasMore),
  );

  async reload(signedIn: boolean): Promise<void> {
    this.signedIn = signedIn;
    this.personal.sort = this.public.sort;
    this.personal.window = this.public.window;
    await Promise.all([this.reloadPublic(), this.reloadPersonal()]);
  }

  async more(): Promise<void> {
    await Promise.all([
      this.shownScope === 'global' && this.public.hasMore ? this.public.more() : undefined,
      this.signedIn && this.personal.hasMore ? this.personal.more() : undefined,
    ]);
  }

  setScope(scope: FeedScope): Promise<void> {
    this.scope = scope;
    keep(KEYS.scope, scope);
    return this.reload(this.signedIn);
  }

  setAudience(audience: HomeAudience): void {
    this.audience = audience;
    keep(KEYS.audience, audience);
  }

  setSort(sort: FeedSort): Promise<void> {
    this.public.sort = sort;
    return this.reload(this.signedIn);
  }

  setWindow(window: FeedWindow): Promise<void> {
    this.public.window = window;
    return this.reload(this.signedIn);
  }

  signal(card: ProjectCard, next: Stance | null): Promise<void> {
    return this.public.signal(card, next);
  }

  private async reloadPublic(): Promise<void> {
    if (this.shownScope === 'global') return this.public.reload();
    this.public.items = [];
    this.public.total = 0;
    this.public.loading = false;
  }

  private async reloadPersonal(): Promise<void> {
    const following = this.signedIn ? (await followCircles()).following : [];
    this.personal.creator = following.join(',');
    if (following.length > 0) return this.personal.reload();
    this.personal.items = [];
    this.personal.total = 0;
    this.personal.loading = false;
  }
}
