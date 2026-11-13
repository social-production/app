<script lang="ts">
  import VoteStrip from '#lib/components/cards/VoteStrip.svelte';
  import { formatCompactCount } from '#lib/domain/count.ts';
  import { countComments } from '#lib/domain/discussions.ts';

  import DiscussionTools from './DiscussionTools.svelte';
  import type { DiscussionState } from './discussionState.svelte.ts';

  /**
   * A post or thread page's actions (ADR 0100): soft pills for the votes, the comment count
   * and Share, then a "Comments" heading with Sort at its right.
   */
  let { state, kind, title }: { state: DiscussionState; kind: 'thread' | 'post'; title: string } =
    $props();

  const count = $derived(countComments(state.comments));
  const label = $derived(`${count} ${count === 1 ? 'comment' : 'comments'}`);
</script>

{#if state.project}
  <div class="item-actions">
    <VoteStrip
      bare
      support={state.project.demand}
      oppose={state.project.opposition}
      mine={state.project.mySignal}
      onSignal={(next) => state.vote(next)}
    />
    <span class="item-pill" role="img" aria-label={label} title={label}>
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
    <DiscussionTools only="share" {kind} {title} bind:sortMode={state.sortMode} />
  </div>
  <div class="comments-head">
    <h2>Comments</h2>
    <DiscussionTools only="sort" {kind} {title} bind:sortMode={state.sortMode} />
  </div>
{/if}
