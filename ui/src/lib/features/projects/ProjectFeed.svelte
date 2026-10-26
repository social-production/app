<script lang="ts">
  import { untrack } from 'svelte';
  import { MediaQuery } from 'svelte/reactivity';

  import PersonalActivityCard from '#lib/components/cards/PersonalActivityCard.svelte';
  import ProjectCard from '#lib/components/cards/ProjectCard.svelte';
  import IconMenuButton from '#lib/components/IconMenuButton.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import type { FeedScope, HomeAudience } from '#lib/domain/homeFeed.ts';
  import type { FeedSort, FeedWindow } from '#lib/domain/projects.ts';

  import { connection } from '#lib/stores/connection.svelte.ts';

  import { HomeFeedState } from './homeFeedState.svelte.ts';

  import '#lib/components/styles/feed.css';

  /**
   * web's PublicFeed (projects only until threads/events land): scope, audience, filter, sort,
   * window, cards. Region joins the scope menu with places (M6).
   */
  let { search = '' }: { search?: string } = $props();

  const feed = new HomeFeedState(() => search);
  const labels = new MediaQuery('(min-width: 720px)');
  const signedIn = $derived(connection.current.status === 'connected');
  const scopes: { value: FeedScope; label: string }[] = [
    { value: 'home', label: 'Home' },
    { value: 'global', label: 'Global' },
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

  // Reload when the search or sign-in changes; the reload reads feed state, which must not
  // re-trigger it.
  $effect(() => {
    void search;
    const viewer = signedIn;
    untrack(() => void feed.reload(viewer));
  });
</script>

<section class="feed-page">
  <section class="toolbar-card">
    <div class="controls-row controls-row-primary">
      <IconMenuButton
        value={feed.shownScope}
        ariaLabel="Choose public feed scope"
        defaultValue="global"
        options={scopes}
        showTriggerLabel={labels.current}
        onChange={(v) => void feed.setScope(v)}
      >
        <ToolbarIcon name={feed.shownScope === 'home' ? 'home' : 'globe'} />
      </IconMenuButton>
      {#if signedIn}
        <IconMenuButton
          value={feed.audience}
          ariaLabel="Choose personal audience"
          options={audiences}
          showTriggerLabel={labels.current}
          onChange={(v) => feed.setAudience(v)}
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
    </div>
  </section>

  <div class="stack">
    {#if feed.error}
      <section class="empty-card"><p>{feed.error}</p></section>
    {:else if feed.loading && feed.rows.length === 0}
      <section class="empty-card"><p>Loading feed...</p></section>
    {:else if feed.rows.length === 0}
      <section class="empty-card">
        <p>
          {search ? 'No projects match this search yet.' : 'No Home items match this filter yet.'}
        </p>
      </section>
    {:else}
      {#each feed.rows as row (row.key)}
        {#if row.source === 'personal'}
          <PersonalActivityCard
            card={row.card}
            authorName={row.card.creatorName || 'Unnamed'}
            onSignal={(next) => feed.signal(row.card, next)}
          />
        {:else}
          <ProjectCard
            card={row.card}
            authorName={row.card.creatorName || 'Unnamed'}
            onSignal={(next) => feed.signal(row.card, next)}
          />
        {/if}
      {/each}
      <InfiniteSentinel disabled={!feed.hasMore || feed.loading} onMore={() => void feed.more()} />
      {#if !feed.hasMore}<p class="end-copy">You're caught up.</p>{/if}
    {/if}
  </div>
</section>
