<script lang="ts">
  import FeedSurface from '#lib/components/cards/FeedSurface.svelte';
  import DiscussionPanel from '#lib/components/discussion/DiscussionPanel.svelte';
  import ThreadOverview from '#lib/components/discussion/ThreadOverview.svelte';

  import { resolve } from '$app/paths';

  import { ThreadState } from './threadState.svelte.ts';

  import '#lib/components/styles/discussion.css';

  /** web's ThreadDetailPage: the thread's overview, then its discussion, on one surface. */
  let { projectId, threadId }: { projectId: string; threadId: string } = $props();

  // The route re-creates this page per thread ({#key}), so reading the ids once is intended.
  // svelte-ignore state_referenced_locally
  const page = new ThreadState(projectId, threadId);

  $effect(() => {
    void page.open();
  });
</script>

<div class="thread-page">
  <section class="page">
    <FeedSurface accent="var(--type-accent-thread)" clamp={false} last>
      {#if page.detail}
        {@const thread = page.detail.thread}
        <ThreadOverview
          {thread}
          authorName={page.names[thread.author] ?? '…'}
          scopeTitle={page.projectTitle || 'Project'}
          scopeHref={resolve('/projects/[id]', { id: projectId })}
          sort={page.sort}
          onSort={(s) => (page.sort = s)}
          onVote={(next) => page.vote(thread.id, next)}
        />
        <DiscussionPanel
          embedded
          comments={page.comments}
          paused={page.paused}
          error={page.writeError}
          onComment={(body) => page.comment(body)}
          onReply={(parent, body) => page.comment(body, parent)}
          onVote={(id, next) => page.vote(id, next)}
        />
      {:else}
        <p class="thread-status">{page.error || 'Loading thread…'}</p>
      {/if}
    </FeedSurface>
  </section>
</div>
