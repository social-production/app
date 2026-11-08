import {
  mergeHomeFeed,
  type FeedScope,
  type HomeAudience,
  type HomeFeedRow,
} from '#lib/domain/homeFeed.ts';
import type { FeedSort, FeedWindow, ProjectCard, Stance } from '#lib/domain/projects.ts';
import type { RegionLink } from '#lib/domain/regionFeed.ts';
import { scopeTag } from '#lib/domain/scopes.ts';
import { communityEventCards } from '#lib/services/closedCommunities.ts';
import { followCircles } from '#lib/services/follows.ts';
import { myAgent } from '#lib/services/people.ts';
import { myScopes } from '#lib/services/scopes.ts';

import { FeedState } from './feedState.svelte.ts';
import { RegionFeedState } from './regionState.svelte.ts';

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
 * personal feed (web-backend's `get_personal_feed`: what the people you follow made, plus your
 * own posts; "Following + popular" adds everyone else's posts and threads). Scope and audience
 * are kept per device (web keeps them in its settings).
 */
export class HomeFeedState {
  scope = $state<FeedScope>(read(KEYS.scope, ['home', 'global', 'region'], 'global'));
  audience = $state<HomeAudience>(read(KEYS.audience, ['following', 'popular'], 'popular'));
  readonly public = new FeedState(() => this.search());
  readonly personal = new FeedState(() => '');
  /** web's Region scope (ADR 0084): its own centre, radius and pages. */
  readonly region = new RegionFeedState(() => ({ sort: this.sort, window: this.window }));
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
  /** Region works signed out too (web); Home needs an account. */
  readonly shownScope = $derived<FeedScope>(
    this.signedIn || this.scope === 'region' ? this.scope : 'global',
  );

  readonly rows = $derived.by((): HomeFeedRow[] => {
    const shown =
      this.shownScope === 'region'
        ? this.region.items
        : this.shownScope === 'home'
          ? [...this.public.items, ...matching(this.communityEvents, this.search())]
          : this.public.items;
    if (!this.signedIn) return mergeHomeFeed(shown, [], 'trending');
    const sort = this.public.sort === 'recent' ? 'recent' : 'trending';
    return mergeHomeFeed(shown, this.personal.items, sort);
  });

  private readonly regional = $derived(this.shownScope === 'region');
  readonly loading = $derived(
    (this.regional ? this.region.loading : this.public.loading) || this.personal.loading,
  );
  readonly error = $derived(
    (this.regional ? this.region.error : this.public.error) ?? this.personal.error,
  );
  readonly hasMore = $derived(
    (this.regional ? this.region.hasMore : this.public.hasMore) ||
      (this.signedIn && this.personal.hasMore),
  );

  async reload(signedIn: boolean): Promise<void> {
    this.signedIn = signedIn;
    this.personal.sort = this.public.sort;
    this.personal.window = this.public.window;
    await Promise.all([this.reloadPublic(), this.reloadPersonal()]);
  }

  async more(): Promise<void> {
    const shown = this.regional ? this.region : this.public;
    await Promise.all([
      shown.hasMore ? shown.more() : undefined,
      this.signedIn && this.personal.hasMore ? this.personal.more() : undefined,
    ]);
  }

  setScope(scope: FeedScope): Promise<void> {
    this.scope = scope;
    keep(KEYS.scope, scope);
    return this.reload(this.signedIn);
  }

  /** The map's "Open as list": Region on the link's place, radius and window. */
  openRegion(link: RegionLink, signedIn: boolean): Promise<void> {
    this.signedIn = signedIn;
    this.region.follow(link);
    const window = ['today', 'week', 'month', 'all'].find((w) => w === link.window);
    if (window) this.public.window = window as FeedWindow;
    return this.setScope('region');
  }

  setAudience(audience: HomeAudience): Promise<void> {
    this.audience = audience;
    keep(KEYS.audience, audience);
    return this.signedIn ? this.reloadPersonal() : Promise.resolve();
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
    if (this.shownScope === 'region') return this.region.reload();
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
    if (!this.signedIn) {
      this.personal.items = [];
      this.personal.total = 0;
      this.personal.loading = false;
      return;
    }
    const [{ following }, me] = await Promise.all([followCircles(), myAgent()]);
    this.personal.creator = following.join(',');
    this.personal.personal = { me, popular: this.audience === 'popular' };
    return this.personal.reload();
  }
}
