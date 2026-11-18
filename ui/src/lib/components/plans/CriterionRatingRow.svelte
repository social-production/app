<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import {
    PLAN_RATING_OPTIONS,
    ratingLabel,
    type PlanCriterion,
    type PlanRating,
  } from '#lib/domain/plans.ts';

  import '#lib/components/styles/plans.css';

  /** One question of a plan review, one per screen (owner, 2026-11-18): five joined targets
   * from 1 to 5; tapping your rating again clears it. The question is the sheet's headline. */
  interface Props {
    criterion: PlanCriterion;
    canVote: boolean;
    onSelect: (rating: PlanRating | null) => void;
  }

  let { criterion, canVote, onSelect }: Props = $props();
</script>

<div class="criterion-row">
  <div class="fives" role="group" aria-label={`Rating scale for ${criterion.label}`}>
    {#each PLAN_RATING_OPTIONS as option (option.value)}
      <button
        class:selected={criterion.mine === option.value}
        class="rating-chip"
        disabled={!canVote}
        title={t(option.label)}
        type="button"
        onclick={() => onSelect(criterion.mine === option.value ? null : option.value)}
        >{option.value}</button
      >
    {/each}
  </div>
  <div class="ends"><span>{t('Strongly oppose')}</span><span>{t('Strongly support')}</span></div>
  <span class="criterion-meta">
    Your rating: {t(ratingLabel(criterion.mine))}
    {#if criterion.count > 0}
      · Avg {criterion.average.toFixed(1)}/5 ({criterion.count})
    {/if}
  </span>
</div>
