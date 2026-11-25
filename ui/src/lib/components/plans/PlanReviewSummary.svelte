<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { criterionShort, ratingLabel, type PlanCriterion } from '#lib/domain/plans.ts';

  /**
   * Your answers on one page before the final vote (Plan review mockup §3): each question in
   * a few words, your answer, and the members' average now that yours is in. Tap a row to
   * change it.
   */
  interface Props {
    criteria: PlanCriterion[];
    onEdit: (index: number) => void;
  }

  let { criteria, onEdit }: Props = $props();
</script>

<div class="plan-answers">
  {#each criteria as c, i (c.id)}
    <button type="button" onclick={() => onEdit(i)}>
      <span>{criterionShort(c)}</span>
      <em class:missing={c.mine === null}
        >{c.mine === null ? t('Answer') : t(ratingLabel(c.mine))}</em
      >
      {#if c.mine !== null && c.count > 0}
        <small>{t('Members {avg} · {n} answered', { avg: c.average.toFixed(1), n: c.count })}</small
        >
      {/if}
    </button>
  {/each}
</div>
<p class="plan-answers-hint">{t('Tap an answer to change it.')}</p>
<p class="outcome">{t('Does this plan meet the need? Your vote is final.')}</p>
