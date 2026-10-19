<script lang="ts">
  import { MediaQuery } from 'svelte/reactivity';

  import ThreadCard from '#lib/components/cards/ThreadCard.svelte';
  import IconMenuButton from '#lib/components/IconMenuButton.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import type { ThreadSort, ThreadWindow } from '#lib/domain/threads.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import CreateThread from './CreateThread.svelte';
  import { ThreadsState } from './threadsState.svelte.ts';

  import '#lib/components/styles/feed.css';

  /**
   * A project's threads, laid out like web's channel/community feed (DirectoryFeedLayout) with
   * threads only: sort and window menus, thread cards, and "New thread" (ADR 0041).
   */
  let { projectId }: { projectId: string } = $props();

  // The project page mounts this once per project, so the first id is the one to keep.
  // svelte-ignore state_referenced_locally
  const threads = new ThreadsState(projectId);
  const labels = new MediaQuery('(min-width: 720px)');
  let creating = $state(false);

  const sorts: { value: ThreadSort; label: string }[] = [
    { value: 'trending', label: 'Trending' },
    { value: 'recent', label: 'Most recent' },
  ];
  const windows: { value: ThreadWindow; label: string }[] = [
    { value: 'today', label: 'Today' },
    { value: 'week', label: 'This week' },
    { value: 'month', label: 'This month' },
    { value: 'all', label: 'All time' },
  ];

  $effect(() => {
    void threads.open();
  });

  async function publish(title: string, body: string): Promise<void> {
    const id = await threads.create(title, body);
    if (!id) return;
    creating = false;
    await goto(resolve('/projects/[id]/threads/[thread]', { id: projectId, thread: id }));
  }
</script>

<section class="feed-page project-threads">
  <section class="toolbar-card">
    <div class="controls-row">
      <IconMenuButton
        value={threads.sort}
        ariaLabel="Sort threads by"
        options={sorts}
        showTriggerLabel={labels.current}
        onChange={(v) => ((threads.sort = v), void threads.reload())}
      >
        <ToolbarIcon name="sort" />
      </IconMenuButton>
      <IconMenuButton
        value={threads.window}
        ariaLabel="Threads time window"
        defaultValue="all"
        options={windows}
        showTriggerLabel={labels.current}
        onChange={(v) => ((threads.window = v), void threads.reload())}
      >
        <ToolbarIcon name="clock" />
      </IconMenuButton>
      {#if !threads.paused}
        <span class="threads-new">
          <RoundPlusButton label="New thread" onclick={() => (creating = true)} />
        </span>
      {/if}
    </div>
  </section>

  <div class="stack">
    {#if threads.error}
      <section class="empty-card"><p>{threads.error}</p></section>
    {:else if !threads.loaded}
      <section class="empty-card"><p>Loading threads...</p></section>
    {:else if threads.items.length === 0}
      <section class="empty-card"><p>No threads yet. Start the first discussion.</p></section>
    {:else}
      {#each threads.items as item (item.id)}
        <ThreadCard
          {projectId}
          {item}
          authorName={threads.names[item.author] ?? '…'}
          onVote={(next) => threads.vote(item, next)}
        />
      {/each}
      <p class="end-copy">You're caught up.</p>
    {/if}
  </div>
</section>

{#if creating}
  <CreateThread
    submitting={threads.creating}
    message={threads.createError}
    onSubmit={(title, body) => void publish(title, body)}
    onClose={() => (creating = false)}
  />
{/if}
