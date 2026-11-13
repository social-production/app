<script lang="ts">
  import VoteStrip from '#lib/components/cards/VoteStrip.svelte';
  import { formatCompactCount } from '#lib/domain/count.ts';
  import { countComments } from '#lib/domain/discussions.ts';

  import DiscussionTools from './DiscussionTools.svelte';
  import type { DiscussionState } from './discussionState.svelte.ts';

  /**
   * A post or thread page's action row (ADR 0100): the feed row's bare actions (ADR 0097) for
   * the item itself: comments, the votes, then Share and Sort.
   */
  let { state, kind, title }: { state: DiscussionState; kind: 'thread' | 'post'; title: string } =
    $props();

  const count = $derived(countComments(state.comments));
  const label = $derived(`${count} ${count === 1 ? 'comment' : 'comments'}`);
</script>

{#if state.project}
  <div class="item-actions">
    <span class="feed-action static" role="img" aria-label={label} title={label}>
      <svg aria-hidden="true" viewBox="0 0 24 24" class="feed-action-icon"
        ><path
          d="M6 5.5h12a1.5 1.5 0 0 1 1.5 1.5v8a1.5 1.5 0 0 1-1.5 1.5h-8l-4 3v-3A1.5 1.5 0 0 1 4.5 15V7A1.5 1.5 0 0 1 6 5.5Z"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linejoin="round"
        /></svg
      >
      <span aria-hidden="true">{formatCompactCount(count)}</span>
    </span>
    <VoteStrip
      bare
      support={state.project.demand}
      oppose={state.project.opposition}
      mine={state.project.mySignal}
      onSignal={(next) => state.vote(next)}
    />
    <DiscussionTools {kind} {title} bind:sortMode={state.sortMode} />
  </div>
{/if}
