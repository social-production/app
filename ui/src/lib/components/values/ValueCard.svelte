<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { IMPORTANCE_OPTIONS, type Importance, type ProjectValue } from '#lib/domain/values.ts';

  import DiscreteScale from './DiscreteScale.svelte';

  import '#lib/components/styles/values.css';

  /** A value as a soft card (Plan, Phase and Feedback Screens, 2): its name and a chip with
   * your rating (or "Rate this"), ten round targets, and everyone's average as a bar. Who
   * added it is in the card's ⋯ sheet. */
  interface Props {
    value: ProjectValue;
    authorName: string;
    canVote: boolean;
    onVote: (importance: Importance) => void;
  }

  let { value, authorName, canVote, onVote }: Props = $props();

  let about = $state(false);
  // The tapped vote shows at once and holds until the reload carries it (web's `heldVote`).
  let heldVote = $state<Importance | null>(null);
  const localVote = $derived(
    heldVote !== null && value.activeImportanceVote !== heldVote
      ? heldVote
      : value.activeImportanceVote,
  );
  const needsVote = $derived(canVote && localVote === 0);
  const average = $derived(value.importanceScore.toFixed(1).replace(/\.0$/, ''));
  const ratings = $derived(`${value.voteCount} rating${value.voteCount === 1 ? '' : 's'}`);
</script>

<article
  class="value-card"
  class:needs-vote={needsVote}
  data-participation-action={needsVote ? 'rate-value' : undefined}
>
  <div class="value-head">
    <strong class="value-title">{value.label}</strong>
    {#if localVote > 0}
      <span class="value-chip mine">You: {localVote}</span>
    {:else if canVote}
      <span class="value-chip need">Rate this</span>
    {/if}
    <button
      class="value-more"
      type="button"
      aria-label={`About ${value.label}`}
      onclick={() => (about = true)}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24"
        ><circle cx="12" cy="5.5" r="1.3" /><circle cx="12" cy="12" r="1.3" /><circle
          cx="12"
          cy="18.5"
          r="1.3"
        /></svg
      >
    </button>
  </div>

  <DiscreteScale
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

  <div
    class="value-average"
    aria-label={value.voteCount ? `Community average: ${average}/10` : 'No community rating yet'}
  >
    <span>Everyone</span>
    <i style={`--share: ${value.voteCount ? value.importanceScore * 10 : 0}%`}></i>
    <span
      >{#if value.voteCount}<b>{average}</b> · {ratings}{:else}No ratings yet{/if}</span
    >
  </div>
</article>

<OverlaySheet
  open={about}
  title={value.label}
  labelledById={`value-about-${value.id}`}
  rootClass="value-sheet"
  onClose={() => (about = false)}
>
  <p class="value-about">
    Added by <strong>{authorName}</strong> · {new Date(value.createdAt / 1000).toLocaleDateString()}
  </p>
</OverlaySheet>
