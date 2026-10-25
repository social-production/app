<script lang="ts">
  import { untrack } from 'svelte';

  import ProjectCard from '#lib/components/cards/ProjectCard.svelte';
  import IconMenuButton from '#lib/components/IconMenuButton.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import InfiniteSentinel from '#lib/components/InfiniteSentinel.svelte';
  import type { FeedSort } from '#lib/domain/projects.ts';

  import { connection } from '#lib/stores/connection.svelte.ts';

  import { FeedState } from './feedState.svelte.ts';

  /** web's profile feed (ProfilePage's toolbar and stack): the person's public items. The app
   * has their projects so far, shown with the public feed's cards (PARITY.md). */
  let { agent }: { agent?: string } = $props();

  // Your own profile (/profile) passes no key: it is the signed-in person's.
  const own = connection.current.status === 'connected' ? connection.current.agent : '';
  // The page is re-created per person ({#key}), so reading `agent` once is intended.
  // svelte-ignore state_referenced_locally
  const feed = new FeedState(() => '', agent ?? own);
  // web's scope menu; "Personal" (followers only) comes with follows.
  const scopes: { value: 'all' | 'public'; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'public', label: 'Public' },
  ];
  let scope = $state<'all' | 'public'>('all');
  const sorts: { value: FeedSort; label: string }[] = [
    { value: 'recent', label: 'Newest' },
    { value: 'top', label: 'Top' },
    { value: 'oldest', label: 'Oldest' },
  ];

  $effect(() => {
    untrack(() => {
      feed.sort = 'recent';
      void feed.reload();
    });
  });
</script>

<section class="toolbar-card">
  <div class="controls-row">
    <IconMenuButton
      value={scope}
      ariaLabel="Filter profile feed"
      defaultValue="all"
      options={scopes}
      onChange={(v) => (scope = v)}
    >
      <ToolbarIcon name={scope === 'public' ? 'globe' : 'filter'} />
    </IconMenuButton>
    <IconMenuButton
      value={feed.sort}
      ariaLabel="Sort profile feed"
      defaultValue="recent"
      options={sorts}
      onChange={(v) => ((feed.sort = v), void feed.reload())}
    >
      <ToolbarIcon name="sort" />
    </IconMenuButton>
  </div>
</section>

<section class="feed-stack">
  {#if feed.error}
    <section class="empty-card"><p>{feed.error}</p></section>
  {:else if feed.loading && feed.items.length === 0}
    <section class="empty-card"><p>Loading activity…</p></section>
  {:else if feed.items.length === 0}
    <section class="empty-card"><p>No activity matches this view yet.</p></section>
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
</section>
