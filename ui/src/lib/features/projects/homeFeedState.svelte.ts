import {
  mergeHomeFeed,
  type FeedScope,
  type HomeAudience,
  type HomeFeedRow,
} from '#lib/domain/homeFeed.ts';
import type { FeedSort, FeedWindow, ProjectCard, Stance } from '#lib/domain/projects.ts';
import { scopeTag } from '#lib/domain/scopes.ts';
import { communityEventCards } from '#lib/services/closedCommunities.ts';
import { followCircles } from '#lib/services/follows.ts';
import { myScopes } from '#lib/services/scopes.ts';

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

/** Your private communities' events matching the search (the index never sees them). */
function matching(cards: ProjectCard[], q: string): ProjectCard[] {
  const needle = q.trim().toLowerCase();
  if (!needle) return cards;
  return cards.filter((c) => `${c.title} ${c.summary}`.toLowerCase().includes(needle));
}

/**
 * web's PublicFeed with its default `combineFeeds` (the app has no setting for it): signed in,
 * the public rows (Global, or Home: items tagged with any channel or community you joined, plus
 * the events listed on your private communities, as web-backend's home feed) merge with the
 * personal feed, which is the projects of people you follow. "Following + popular"
 * adds discovery posts and threads on web; the app has none yet. Scope and audience are kept
 * per device (web keeps them in its settings).
 */
export class HomeFeedState {
  scope = $state<FeedScope>(read(KEYS.scope, ['home', 'global'], 'global'));
  audience = $state<HomeAudience>(read(KEYS.audience, ['following', 'popular'], 'popular'));
  readonly public = new FeedState(() => this.search());
  readonly personal = new FeedState(() => '');
  private signedIn = $state(false);
  /** Home: events listed on your private communities (ADR 0069), read from their networks. */
  private communityEvents = $state<ProjectCard[]>([]);

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
    const shown =
      this.shownScope === 'home'
        ? [...this.public.items, ...matching(this.communityEvents, this.search())]
        : this.public.items;
    if (!this.signedIn) return mergeHomeFeed(shown, [], 'trending');
    const sort = this.public.sort === 'recent' ? 'recent' : 'trending';
    return mergeHomeFeed(shown, this.personal.items, sort);
  });

  readonly loading = $derived(this.public.loading || this.personal.loading);
  readonly error = $derived(this.public.error ?? this.personal.error);
  readonly hasMore = $derived(this.public.hasMore || (this.signedIn && this.personal.hasMore));

  async reload(signedIn: boolean): Promise<void> {
    this.signedIn = signedIn;
    this.personal.sort = this.public.sort;
    this.personal.window = this.public.window;
    await Promise.all([this.reloadPublic(), this.reloadPersonal()]);
  }

  async more(): Promise<void> {
    await Promise.all([
      this.public.hasMore ? this.public.more() : undefined,
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
    this.communityEvents = [];
    if (this.shownScope === 'global') {
      this.public.tag = undefined;
      return this.public.reload();
    }
    this.public.loading = true;
    const scopes = await myScopes().catch(() => []);
    const open = scopes.filter((s) => !s.isPrivate);
    const closed = scopes.filter((s) => s.isPrivate);
    this.public.tag = open.map((s) => scopeTag(s.kind, s.slug)).join(',');
    await Promise.all([
      open.length > 0 ? this.public.reload() : this.clearPublic(),
      this.loadCommunityEvents(closed.map((s) => s.id)),
    ]);
  }

  /** web-backend's home feed: no memberships, no public rows. */
  private clearPublic(): void {
    this.public.items = [];
    this.public.total = 0;
    this.public.loading = false;
  }

  private async loadCommunityEvents(communities: string[]): Promise<void> {
    const found = await Promise.allSettled(communities.map((c) => communityEventCards(c)));
    const cards = found.flatMap((r) => (r.status === 'fulfilled' ? r.value : []));
    // An event is listed on one community, but a stale listing may repeat it.
    this.communityEvents = cards.filter((c, i) => cards.findIndex((o) => o.id === c.id) === i);
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
