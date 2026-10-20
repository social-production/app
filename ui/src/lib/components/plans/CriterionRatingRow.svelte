<script lang="ts">
  import {
    PLAN_RATING_OPTIONS,
    ratingLabel,
    type PlanCriterion,
    type PlanRating,
  } from '#lib/domain/plans.ts';

  import '#lib/components/styles/plans.css';

  /** web's PlanCriterionRatingRow: one rubric question rated 1–5; tapping your rating clears it. */
  interface Props {
    criterion: PlanCriterion;
    canVote: boolean;
    onSelect: (rating: PlanRating | null) => void;
  }

  let { criterion, canVote, onSelect }: Props = $props();
</script>

<div class="criterion-row">
  <div class="criterion-copy">
    <span class="criterion-kind">{criterion.kind === 'Value' ? 'Shared value' : 'Criterion'}</span>
    <strong>{criterion.label}</strong>
    <span class="criterion-meta">
      Your rating: {ratingLabel(criterion.mine)}
      {#if criterion.count > 0}
        · Avg {criterion.average.toFixed(1)}/5 ({criterion.count})
      {/if}
    </span>
  </div>

  <div class="rating-row" role="group" aria-label={`Rating scale for ${criterion.label}`}>
    {#each PLAN_RATING_OPTIONS as option (option.value)}
      <button
        class:selected={criterion.mine === option.value}
        class="rating-chip"
        disabled={!canVote}
        title={option.label}
        type="button"
        onclick={() => onSelect(criterion.mine === option.value ? null : option.value)}
      >
        <span class="rating-value">{option.value}</span>
        <span class="rating-label">{option.label}</span>
      </button>
    {/each}
  </div>
</div>
