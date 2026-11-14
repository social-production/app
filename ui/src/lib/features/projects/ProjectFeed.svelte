<script lang="ts">
  import { untrack } from 'svelte';

  import { page } from '$app/state';
  import { resolve } from '$app/paths';
  import type { ResolvedPathname } from '$app/types';
  import PersonalSpaceCard from '#lib/components/cards/PersonalSpaceCard.svelte';
  import ProjectActivityCard from '#lib/components/cards/ProjectActivityCard.svelte';
  import SpaceCard from '#lib/components/cards/SpaceCard.svelte';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import { regionLink, type RegionActivity } from '#lib/domain/regionFeed.ts';

  import { connection } from '#lib/stores/connection.svelte.ts';

  import FeedHeader, { type FeedFilter } from './FeedHeader.svelte';
  import { HomeFeedState } from './homeFeedState.svelte.ts';

  import '#lib/components/styles/feed.css';

  /**
   * web's PublicFeed (projects only until threads/events land): scope, audience, filter, sort,
   * window, cards; Region (ADR 0084, 0085) has its own centre, radius and online list.
   */
  let { search = '' }: { search?: string } = $props();

  const feed = new HomeFeedState(() => search);
  const signedIn = $derived(connection.current.status === 'connected');
  let filter = $state<FeedFilter>('all');

  // The Region centre is read once the connection settles (your default place is kept under
  // your key). The map's "Open as list" opens Region on its place (also while the feed shows).
  let hydrated = $state(false);
  let followed = '';
  $effect(() => {
    const current = connection.current;
    if (hydrated || current.status === 'connecting') return;
    untrack(() => {
      feed.region.hydrate(current.status === 'connected' ? current.agent : null);
      hydrated = true;
    });
  });

  $effect(() => {
    void search;
    const viewer = signedIn;
    const link = regionLink(page.url.searchParams);
    const key = link ? page.url.search : '';
    if (!hydrated) return;
    untrack(() => {
      if (link && key !== followed) {
        followed = key;
        void feed.openRegion(link, viewer);
      } else void feed.reload(viewer);
    });
  });

  /** An activity's project page, with web's `?activity=`. */
  const activityHref = (a: RegionActivity): ResolvedPathname =>
    `${resolve('/projects/[id]', { id: a.parentId })}?activity=${a.id}` as ResolvedPathname;
</script>

<section class="feed-page">
  <FeedHeader {feed} {signedIn} bind:filter />

  <div class="stack">
    {#if feed.error}
      <section class="empty-card"><p>{feed.error}</p></section>
    {:else if feed.loading && feed.rows.length === 0}
      <section class="empty-card"><p>Loading feed...</p></section>
    {:else if feed.shownScope === 'region' && !feed.region.hasCenter}
      <section class="empty-card">
        <p>Choose a place or enable device location to load the region feed.</p>
      </section>
    {:else if feed.rows.length === 0}
      <section class="empty-card">
        <p>
          {search ? 'No projects match this search yet.' : 'No Home items match this filter yet.'}
        </p>
      </section>
    {:else}
      {#each feed.rows as row (row.key)}
        {#if 'activity' in row}
          <ProjectActivityCard item={row.activity} href={activityHref(row.activity)} />
        {:else if row.source === 'personal'}
          <PersonalSpaceCard card={row.card} onSignal={(next) => feed.signal(row.card, next)} />
        {:else}
          <SpaceCard card={row.card} onSignal={(next) => feed.signal(row.card, next)} />
        {/if}
      {/each}
      <InfiniteSentinel disabled={!feed.hasMore || feed.loading} onMore={() => void feed.more()} />
      {#if !feed.hasMore}<p class="end-copy">You're caught up.</p>{/if}
    {/if}
  </div>
</section>
