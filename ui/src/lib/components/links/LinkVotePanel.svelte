<script lang="ts">
  import type { VoteChoice } from '#lib/domain/governance.ts';
  import { resultNote, voteRequirement, voteSummary, type LinkSide } from '#lib/domain/links.ts';

  /**
   * One project's side of a link vote (the vote panel web repeats in each links section).
   * Votes are final here (ADR 0033), so the chosen button stays marked and both disable.
   */
  interface Props {
    side: LinkSide;
    yesLabel?: string;
    noLabel?: string;
    danger?: boolean;
    pending?: boolean;
    showRequirement?: boolean;
    onVote?: ((side: LinkSide, choice: VoteChoice) => void) | undefined;
  }

  let {
    side,
    yesLabel = 'Approve',
    noLabel = 'Oppose',
    danger = false,
    pending = false,
    showRequirement = true,
    onVote,
  }: Props = $props();

  const half = $derived(side.half);
  const voted = $derived(half?.myVote ?? null);
</script>

<section class="vote-panel">
  <div class="vote-head"><strong>{side.projectTitle}</strong></div>
  <p>{voteSummary(half)}</p>
  {#if showRequirement && half?.status === 'Open'}
    <p class="vote-requirement">{voteRequirement(half)}</p>
  {:else}
    <p class="vote-note">{resultNote(half)}</p>
  {/if}
  {#if onVote && (side.canVote || voted)}
    <div class="vote-buttons">
      <button
        class:active={voted === 'Yes'}
        class:danger
        class="vote-button"
        disabled={pending || !side.canVote}
        type="button"
        onclick={() => onVote(side, 'Yes')}>{yesLabel}</button
      >
      <button
        class:active={voted === 'No'}
        class="vote-button"
        disabled={pending || !side.canVote}
        type="button"
        onclick={() => onVote(side, 'No')}>{noLabel}</button
      >
    </div>
  {/if}
</section>
