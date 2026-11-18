<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { PlanCriterion, PlanRating } from '#lib/domain/plans.ts';

  import CriterionRatingRow from './CriterionRatingRow.svelte';

  /** A plan review's questions (Plan, Phase and Feedback Screens, 3): each says which
   * question it is, the project's values say "Shared value". Ratings can change until the
   * final vote, which is final (ADR 0033). */
  interface Props {
    criteria: PlanCriterion[];
    canVote: boolean;
    onRate: (criterion: string, rating: PlanRating | null) => void;
  }

  let { criteria, canVote, onRate }: Props = $props();

  const questions = $derived(criteria.filter((c) => c.kind !== 'Value').length);
  const kicker = (c: PlanCriterion, index: number): string =>
    c.kind === 'Value' ? 'Shared value' : `Question ${index + 1} of ${questions}`;
</script>

<section class="assess-panel" data-participation-action="assess-plan">
  {#if !canVote}
    <p class="assess-blocked">{t('Only members can rate this plan. Join to take part.')}</p>
  {/if}
  <div class="criterion-stack">
    {#each criteria as entry, index (entry.id)}
      <CriterionRatingRow
        criterion={entry}
        kicker={kicker(entry, index)}
        {canVote}
        onSelect={(rating) => onRate(entry.id, rating)}
      />
    {/each}
  </div>
</section>
