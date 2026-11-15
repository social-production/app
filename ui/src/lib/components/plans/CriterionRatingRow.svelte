<script lang="ts">
  import {
    PLAN_RATING_OPTIONS,
    ratingLabel,
    type PlanCriterion,
    type PlanRating,
  } from '#lib/domain/plans.ts';

  import '#lib/components/styles/plans.css';

  /** One question of a plan review (Plan, Phase and Feedback Screens, 3): which question it
   * is, the question, and five large targets; tapping your rating again clears it. */
  interface Props {
    criterion: PlanCriterion;
    /** "Question 2 of 4", or "Shared value" for one of the project's values. */
    kicker: string;
    canVote: boolean;
    onSelect: (rating: PlanRating | null) => void;
  }

  let { criterion, kicker, canVote, onSelect }: Props = $props();
</script>

<div class="criterion-row">
  <small class="criterion-kind">{kicker}</small>
  <strong
    >{criterion.kind === 'Value'
      ? `How well does it satisfy "${criterion.label}"?`
      : criterion.label}</strong
  >
  <div class="fives" role="group" aria-label={`Rating scale for ${criterion.label}`}>
    {#each PLAN_RATING_OPTIONS as option (option.value)}
      <button
        class:selected={criterion.mine === option.value}
        class="rating-chip"
        disabled={!canVote}
        title={option.label}
        type="button"
        onclick={() => onSelect(criterion.mine === option.value ? null : option.value)}
        >{option.value}</button
      >
    {/each}
  </div>
  <div class="ends"><span>Strongly oppose</span><span>Strongly support</span></div>
  <span class="criterion-meta">
    Your rating: {ratingLabel(criterion.mine)}
    {#if criterion.count > 0}
      · Avg {criterion.average.toFixed(1)}/5 ({criterion.count})
    {/if}
  </span>
</div>
