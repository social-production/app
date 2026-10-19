<script lang="ts">
  import { IMPORTANCE_OPTIONS, type Importance, type ProjectValue } from '#lib/domain/values.ts';

  import DiscreteScale from './DiscreteScale.svelte';

  import '#lib/components/styles/values.css';

  /** web's ProjectValueCard: a value, its 1–10 importance scale and the community average. */
  interface Props {
    value: ProjectValue;
    authorName: string;
    canVote: boolean;
    onVote: (importance: Importance) => void;
  }

  let { value, authorName, canVote, onVote }: Props = $props();

  // The tapped vote shows at once and holds until the reload carries it (web's `heldVote`).
  let heldVote = $state<Importance | null>(null);
  const localVote = $derived(
    heldVote !== null && value.activeImportanceVote !== heldVote
      ? heldVote
      : value.activeImportanceVote,
  );

  const hasUserVote = $derived(localVote > 0);
  const needsVote = $derived(canVote && !hasUserVote);
  const averageLabel = $derived(
    value.voteCount === 0
      ? 'No community rating yet'
      : `Community average: ${value.importanceScore.toFixed(1).replace(/\.0$/, '')}/10`,
  );
  const userVoteLabel = $derived(
    hasUserVote ? `Your vote: ${localVote}/10` : 'Vote to rate this value',
  );
  const voteLabel = $derived(`${value.voteCount} vote${value.voteCount === 1 ? '' : 's'}`);
</script>

<article
  class="value-card"
  class:needs-vote={needsVote}
  data-participation-action={needsVote ? 'rate-value' : undefined}
>
  <strong class="value-title">{value.label}</strong>

  {#if needsVote}
    <p class="value-prompt">Tap 1 if this does not matter, 10 if it is required.</p>
  {/if}

  <DiscreteScale
    averageValue={value.voteCount > 0 ? value.importanceScore : 0}
    disabled={!canVote}
    leftLabel="Unnecessary"
    onSelect={(selected) => {
      heldVote = selected as Importance;
      onVote(selected as Importance);
    }}
    options={IMPORTANCE_OPTIONS}
    rightLabel="Required"
    selectedValue={localVote}
  />

  <div class="value-footer">
    <span class="value-footer-start">
      <span>{userVoteLabel}</span>
      {#if value.voteCount > 0}
        <span>{averageLabel}</span>
      {/if}
    </span>
    <span class="value-footer-end">
      <span>{voteLabel}</span>
      <span>{authorName}</span>
    </span>
  </div>
</article>
