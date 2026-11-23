<script lang="ts">
  import { untrack } from 'svelte';

  import { page } from '$app/state';
  import { resolve } from '$app/paths';
  import type { ResolvedPathname } from '$app/types';
  import LoadingLine from '#lib/components/loading/LoadingLine.svelte';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import CommentActivityCard from '#lib/components/cards/CommentActivityCard.svelte';
  import PersonalSpaceCard from '#lib/components/cards/PersonalSpaceCard.svelte';
  import ProjectActivityCard from '#lib/components/cards/ProjectActivityCard.svelte';
  import SpaceCard from '#lib/components/cards/SpaceCard.svelte';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import { regionLink, type RegionActivity } from '#lib/domain/regionFeed.ts';

  import { connection } from '#lib/stores/connection.svelte.ts';

  import EmptyFeedEnd from '#lib/components/empty/FeedEnd.svelte';
  import { keepRow, type FeedType } from '#lib/domain/feedTypes.ts';
  import { keepFeedFilters, readFeedFilters } from '#lib/stores/feedFilters.ts';

  import FeedHeader from './FeedHeader.svelte';
  import HomeFeedEmpty from './HomeFeedEmpty.svelte';
  import { HomeFeedState } from './homeFeedState.svelte.ts';

  import '#lib/components/styles/feed.css';

  /**
   * web's PublicFeed (projects only until threads/events land): scope, audience, filter, sort,
   * window, cards; Region (ADR 0084, 0085) has its own centre, radius and online list.
   */
  let { search = '' }: { search?: string } = $props();

  const feed = new HomeFeedState(() => search);
  const signedIn = $derived(connection.current.status === 'connected');
  // The filters last across reloads, on this device.
  const saved = readFeedFilters('home', { type: 'all', sort: 'trending', window: 'all' });
  feed.public.sort = saved.sort;
  feed.public.window = saved.window;
  let filter = $state<FeedType>(saved.type);
  $effect(() => keepFeedFilters('home', { type: filter, sort: feed.sort, window: feed.window }));
  let sheetOpen = $state(false);
  const rows = $derived(feed.rows.filter((row) => keepRow(row, filter)));
  const narrowed = $derived(filter !== 'all' || feed.window !== 'all');

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

  /** "Maya Kim proposed this" over a card by someone you follow (ADR 0107). */
  // Home and Global name the person you follow over their cards, with their picture (Profiles
  // and Comments §1); Region shows the plain public card.

  /** An activity's project page, with web's `?activity=`. */
  const activityHref = (a: RegionActivity): ResolvedPathname =>
    `${resolve('/projects/[id]', { id: a.parentId })}?activity=${a.id}` as ResolvedPathname;
</script>

<section class="feed-page">
  <FeedHeader
    {feed}
    {signedIn}
    guest={connection.current.status === 'signed-out'}
    bind:filter
    bind:open={sheetOpen}
  />

  <div class="stack">
    {#if feed.loading && rows.length === 0 && !feed.error}
      <LoadingLine /><SkeletonRows count={4} />
    {:else if feed.error || rows.length === 0 || (feed.shownScope === 'region' && !feed.region.hasCenter)}
      <HomeFeedEmpty
        error={feed.error}
        scope={feed.shownScope}
        hasPlace={feed.region.hasCenter}
        {narrowed}
        {search}
        onChoosePlace={() => (sheetOpen = true)}
        onClear={() => ((filter = 'all'), void feed.setWindow('all'))}
      />
    {:else}
      {#each rows as row (row.key)}
        {#if 'activity' in row}
          <ProjectActivityCard item={row.activity} href={activityHref(row.activity)} />
        {:else if 'comment' in row}
          <CommentActivityCard comment={row.comment} />
        {:else if row.source === 'personal'}
          <PersonalSpaceCard card={row.card} onSignal={(next) => feed.signal(row.card, next)} />
        {:else}
          <SpaceCard card={row.card} onSignal={(next) => feed.signal(row.card, next)} />
        {/if}
      {/each}
      <InfiniteSentinel disabled={!feed.hasMore || feed.loading} onMore={() => void feed.more()} />
      {#if !feed.hasMore}<EmptyFeedEnd />{/if}
    {/if}
  </div>
</section>
