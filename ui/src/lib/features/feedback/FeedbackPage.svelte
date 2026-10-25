<script lang="ts">
  import { MediaQuery } from 'svelte/reactivity';

  import FeedbackCard from '#lib/components/feedback/FeedbackCard.svelte';
  import FeedbackComposeSheet from '#lib/components/feedback/FeedbackComposeSheet.svelte';
  import FeedbackDetailSheet from '#lib/components/feedback/FeedbackDetailSheet.svelte';
  import IconMenuButton from '#lib/components/IconMenuButton.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import {
    emptyFeedbackCopy,
    shownFeedback,
    type FeedbackFilter,
    type FeedbackSort,
  } from '#lib/domain/feedback.ts';

  import { FeedbackState } from './feedbackState.svelte.ts';

  import '#lib/components/styles/feedback.css';

  /** web's FeedbackPage: bugs and suggestions with votes, filter and sort, compose and detail. */
  const filterOptions: { value: FeedbackFilter; label: string }[] = [
    { value: 'all', label: 'All feedback' },
    { value: 'bugs', label: 'Bugs' },
    { value: 'suggestions', label: 'Suggestions' },
  ];
  const sortOptions: { value: FeedbackSort; label: string }[] = [
    { value: 'trending', label: 'Trending' },
    { value: 'recent', label: 'Most recent' },
  ];

  const fb = new FeedbackState();
  const labels = new MediaQuery('(min-width: 720px)');
  let filter = $state<FeedbackFilter>('all');
  let sort = $state<FeedbackSort>('trending');
  const items = $derived(shownFeedback(fb.items, filter, sort, Date.now() * 1000));

  $effect(() => {
    void fb.load();
  });
</script>

<section class="feedback-page">
  <div class="page-hero">
    <div class="hero-copy">
      <p class="eyebrow">Platform feedback</p>
      <h1>Feedback</h1>
      <p class="hero-body">
        Track bugs, share suggestions, and vote on what the platform should fix or improve next. The
        code is on <a href="https://github.com/social-production" rel="noreferrer" target="_blank"
          >GitHub</a
        >.
      </p>
    </div>
    <button class="create-button" type="button" onclick={() => fb.compose()}>
      <ToolbarIcon name="plus" />
      <span>Add feedback</span>
    </button>
  </div>

  <div class="toolbar">
    <IconMenuButton
      value={filter}
      ariaLabel="Filter feedback"
      defaultValue="all"
      options={filterOptions}
      showTriggerLabel={labels.current}
      onChange={(v) => (filter = v)}
    >
      <ToolbarIcon name="filter" />
    </IconMenuButton>
    <IconMenuButton
      value={sort}
      ariaLabel="Sort feedback"
      options={sortOptions}
      showTriggerLabel={labels.current}
      onChange={(v) => (sort = v)}
    >
      <ToolbarIcon name={sort === 'recent' ? 'clock' : 'sort'} />
    </IconMenuButton>
  </div>

  {#if fb.error}
    <div class="status-card warning">{fb.error}</div>
  {:else if fb.loading}
    <div class="status-card">Loading feedback…</div>
  {:else if items.length === 0}
    <div class="status-card">{emptyFeedbackCopy(filter)}</div>
  {:else}
    <div class="feedback-list">
      {#each items as item (item.id)}
        <FeedbackCard
          {item}
          pendingVote={fb.voting.includes(item.id)}
          onOpen={() => void fb.open(item.id)}
          onVote={(vote) => void fb.vote(item.id, vote)}
        />
      {/each}
    </div>
  {/if}
</section>

<FeedbackComposeSheet
  open={fb.composeOpen}
  pending={fb.composePending}
  message={fb.composeMessage}
  onSubmit={(input) => void fb.submit(input)}
  onClose={() => (fb.composeOpen = false)}
/>

<FeedbackDetailSheet
  item={fb.detail}
  authorName={fb.detail ? (fb.names[fb.detail.author] ?? 'Someone') : ''}
  pendingVote={fb.detail ? fb.voting.includes(fb.detail.id) : false}
  onVote={(vote) => fb.detail && void fb.vote(fb.detail.id, vote)}
  onClose={() => (fb.detailId = null)}
/>
