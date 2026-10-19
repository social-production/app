<script lang="ts">
  import { untrack } from 'svelte';
  import { MediaQuery } from 'svelte/reactivity';

  import ProjectCard from '#lib/components/cards/ProjectCard.svelte';
  import IconMenuButton from '#lib/components/IconMenuButton.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import type { FeedSort, FeedWindow } from '#lib/domain/projects.ts';

  import { FeedState } from './feedState.svelte.ts';

  import '#lib/components/styles/feed.css';

  /** web's PublicFeed (projects only until threads/events land): sort, window, cards. */
  let { search = '' }: { search?: string } = $props();

  const feed = new FeedState(() => search);
  const labels = new MediaQuery('(min-width: 720px)');
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

  // Reload when the search changes; the reload reads feed state, which must not re-trigger it.
  $effect(() => {
    void search;
    untrack(() => void feed.reload());
  });
</script>

<section class="feed-page">
  <section class="toolbar-card">
    <div class="controls-row controls-row-primary">
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
        onChange={(v) => ((feed.sort = v), void feed.reload())}
      >
        <ToolbarIcon name="sort" />
      </IconMenuButton>
      <IconMenuButton
        value={feed.window}
        ariaLabel="Public feed time window"
        defaultValue="all"
        options={windows}
        showTriggerLabel={labels.current}
        onChange={(v) => ((feed.window = v), void feed.reload())}
      >
        <ToolbarIcon name="clock" />
      </IconMenuButton>
    </div>
  </section>

  <div class="stack">
    {#if feed.error}
      <section class="empty-card"><p>{feed.error}</p></section>
    {:else if feed.loading && feed.items.length === 0}
      <section class="empty-card"><p>Loading feed...</p></section>
    {:else if feed.items.length === 0}
      <section class="empty-card">
        <p>
          {search ? 'No projects match this search yet.' : 'No public items match this filter yet.'}
        </p>
      </section>
    {:else}
      {#each feed.items as card (card.id)}
        <ProjectCard
          {card}
          authorName={card.creatorName || 'Unnamed'}
          onSignal={(next) => feed.signal(card, next)}
        />
      {/each}
      <InfiniteSentinel disabled={!feed.hasMore || feed.loading} onMore={() => void feed.more()} />
      {#if !feed.hasMore}<p class="end-copy">You're caught up.</p>{/if}
    {/if}
  </div>
</section>
