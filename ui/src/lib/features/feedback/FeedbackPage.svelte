<script lang="ts">
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import FeedbackCard from '#lib/components/feedback/FeedbackCard.svelte';
  import FeedbackComposeSheet from '#lib/components/feedback/FeedbackComposeSheet.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import {
    emptyFeedbackCopy,
    shownFeedback,
    type FeedbackFilter,
    type FeedbackSort,
  } from '#lib/domain/feedback.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import { FeedbackState } from './feedbackState.svelte.ts';

  import '#lib/components/styles/feedback.css';

  /** web's FeedbackPage: bugs and suggestions with votes, compose and detail. The filter and
   * the order are in view (Plan, Phase and Feedback Screens, 7), not behind icons. */
  const filterOptions: { value: FeedbackFilter; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'bugs', label: 'Bugs' },
    { value: 'suggestions', label: 'Suggestions' },
  ];
  const sortOptions: { value: FeedbackSort; label: string }[] = [
    { value: 'trending', label: 'Trending' },
    { value: 'recent', label: 'Most recent' },
  ];

  const fb = new FeedbackState();
  let filter = $state<FeedbackFilter>('all');
  let sort = $state<FeedbackSort>('trending');
  const items = $derived(shownFeedback(fb.items, filter, sort, Date.now() * 1000));

  $effect(() => {
    void fb.load();
  });
</script>

<section class="feedback-page">
  <header class="feedback-head">
    <h1>Feedback</h1>
    <div class="feedback-seg" role="radiogroup" aria-label="Filter feedback">
      {#each filterOptions as option (option.value)}
        <button
          role="radio"
          aria-checked={filter === option.value}
          class:on={filter === option.value}
          type="button"
          onclick={() => (filter = option.value)}>{option.label}</button
        >
      {/each}
    </div>
  </header>
  <div class="feedback-tabs" role="tablist" aria-label="Sort feedback">
    {#each sortOptions as option (option.value)}
      <button
        role="tab"
        aria-selected={sort === option.value}
        class:on={sort === option.value}
        type="button"
        onclick={() => (sort = option.value)}>{option.label}</button
      >
    {/each}
  </div>

  {#if fb.error}
    <p class="feedback-note warning">{fb.error}</p>
  {:else if fb.loading}
    <SkeletonRows count={3} />
  {:else if items.length === 0}
    <p class="feedback-note">{emptyFeedbackCopy(filter)}</p>
  {:else}
    <div class="feedback-list">
      {#each items as item (item.id)}
        <FeedbackCard
          {item}
          pendingVote={fb.voting.includes(item.id)}
          onOpen={() => void goto(resolve('/feedback/[id]', { id: item.id }))}
          onVote={(vote) => void fb.vote(item.id, vote)}
        />
      {/each}
    </div>
  {/if}
  <p class="feedback-source">
    The code is on <a href="https://github.com/social-production" rel="noreferrer" target="_blank"
      >GitHub</a
    >.
  </p>

  <button class="feedback-fab" type="button" onclick={() => fb.compose()}>
    <ToolbarIcon name="plus" />
    <span>Add feedback</span>
  </button>
</section>

<FeedbackComposeSheet
  open={fb.composeOpen}
  pending={fb.composePending}
  message={fb.composeMessage}
  onSubmit={(input) => void fb.submit(input)}
  onClose={() => (fb.composeOpen = false)}
/>
