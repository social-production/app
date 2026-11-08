<script lang="ts">
  import { untrack } from 'svelte';
  import { MediaQuery } from 'svelte/reactivity';

  import { page } from '$app/state';
  import { resolve } from '$app/paths';
  import type { ResolvedPathname } from '$app/types';
  import PersonalSpaceCard from '#lib/components/cards/PersonalSpaceCard.svelte';
  import ProjectActivityCard from '#lib/components/cards/ProjectActivityCard.svelte';
  import SpaceCard from '#lib/components/cards/SpaceCard.svelte';
  import IconMenuButton from '#lib/components/IconMenuButton.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import type { FeedScope, HomeAudience } from '#lib/domain/homeFeed.ts';
  import type { FeedSort, FeedWindow } from '#lib/domain/projects.ts';
  import { regionLink, type RegionActivity } from '#lib/domain/regionFeed.ts';

  import { connection } from '#lib/stores/connection.svelte.ts';

  import { HomeFeedState } from './homeFeedState.svelte.ts';
  import RegionControls from './RegionControls.svelte';

  import '#lib/components/styles/feed.css';

  /**
   * web's PublicFeed (projects only until threads/events land): scope, audience, filter, sort,
   * window, cards; Region (ADR 0084, 0085) has its own centre, radius and online list.
   */
  let { search = '' }: { search?: string } = $props();

  const feed = new HomeFeedState(() => search);
  const labels = new MediaQuery('(min-width: 720px)');
  const signedIn = $derived(connection.current.status === 'connected');
  const scopes: { value: FeedScope; label: string }[] = [
    { value: 'home', label: 'Home' },
    { value: 'global', label: 'Global' },
    { value: 'region', label: 'Region' },
  ];
  const audiences: { value: HomeAudience; label: string }[] = [
    { value: 'following', label: 'Following only' },
    { value: 'popular', label: 'Following + popular' },
  ];
  // web's filter menu; threads, events and help requests join it as they land (M4–M5).
  const filters: { value: 'all' | 'projects'; label: string }[] = [
    { value: 'all', label: 'All items' },
    { value: 'projects', label: 'Projects' },
  ];
  let filter = $state<'all' | 'projects'>('all');
  const sorts: { value: FeedSort; label: string }[] = [
    { value: 'trending', label: 'Trending' },
    { value: 'recent', label: 'Most recent' },
  ];
  const windows: { value: FeedWindow; label: string }[] = [
    { value: 'today', label: 'Today' },
    { value: 'week', label: 'This week' },
    { value: 'month', label: 'This month' },
    { value: 'all', label: 'All time' },
  ];

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
  const scopeIcon = $derived(
    feed.shownScope === 'home' ? 'home' : feed.shownScope === 'region' ? 'map-pin' : 'globe',
  );
</script>

<section class="feed-page">
  <section class="toolbar-card" class:toolbar-card-region={feed.shownScope === 'region'}>
    <div class="controls-row controls-row-primary">
      <IconMenuButton
        value={feed.shownScope}
        ariaLabel="Choose public feed scope"
        defaultValue="global"
        options={scopes}
        showTriggerLabel={labels.current && feed.shownScope !== 'region'}
        onChange={(v) => void feed.setScope(v)}
      >
        <ToolbarIcon name={scopeIcon} />
      </IconMenuButton>
      {#if signedIn}
        <IconMenuButton
          value={feed.audience}
          ariaLabel="Choose personal audience"
          options={audiences}
          showTriggerLabel={labels.current}
          onChange={(v) => void feed.setAudience(v)}
        >
          <ToolbarIcon name={feed.audience === 'following' ? 'people' : 'trending'} />
        </IconMenuButton>
      {/if}
      <IconMenuButton
        value={filter}
        ariaLabel="Filter public feed"
        defaultValue="all"
        options={filters}
        showTriggerLabel={labels.current}
        onChange={(v) => (filter = v)}
      >
        <ToolbarIcon name="filter" />
      </IconMenuButton>
      <IconMenuButton
        value={feed.sort}
        ariaLabel="Sort public feed by"
        options={sorts}
        showTriggerLabel={labels.current}
        onChange={(v) => void feed.setSort(v)}
      >
        <ToolbarIcon name="sort" />
      </IconMenuButton>
      <IconMenuButton
        value={feed.window}
        ariaLabel="Public feed time window"
        defaultValue="all"
        options={windows}
        showTriggerLabel={labels.current}
        onChange={(v) => void feed.setWindow(v)}
      >
        <ToolbarIcon name="clock" />
      </IconMenuButton>
      {#if feed.shownScope === 'region'}<RegionControls region={feed.region} />{/if}
    </div>
    {#if feed.shownScope === 'region'}
      {#if feed.region.message}
        <p class="inline-alert" role="alert">{feed.region.message}</p>
      {/if}
      {#if feed.region.suggestions.length > 0}
        <ul class="suggestions">
          {#each feed.region.suggestions as place, i (i)}
            <li>
              <button
                type="button"
                onmousedown={(e) => e.preventDefault()}
                onclick={() => void feed.region.select(place)}>{place.displayLabel}</button
              >
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </section>

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
