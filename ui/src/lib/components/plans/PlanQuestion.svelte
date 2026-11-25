<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { PLAN_RATING_OPTIONS, type PlanCriterion, type PlanRating } from '#lib/domain/plans.ts';

  /**
   * One question of a plan review (Plan review mockup §2, owner 2026-11-25): the plan it is
   * about, five answers in words (1–5 underneath), and the members' answers only once yours
   * is in. Tapping an answer moves on by itself.
   */
  interface Props {
    criterion: PlanCriterion;
    planTitle: string;
    canVote: boolean;
    onAnswer: (rating: PlanRating) => void;
  }

  let { criterion, planTitle, canVote, onAnswer }: Props = $props();
</script>

<div class="plan-q">
  <p class="plan-q-about">{t('Plan')} · <b>{planTitle}</b></p>
  <div class="plan-q-choices" role="radiogroup" aria-label={t('Your answer')}>
    {#each PLAN_RATING_OPTIONS as option (option.value)}
      <button
        class="plan-q-choice"
        class:selected={criterion.mine === option.value}
        type="button"
        role="radio"
        aria-checked={criterion.mine === option.value}
        disabled={!canVote}
        onclick={() => onAnswer(option.value)}
        ><span class="plan-q-dot">{option.value}</span>{t(option.label)}</button
      >
    {/each}
  </div>
  <p class="plan-q-hint">
    {#if criterion.mine === null}
      {t("Others' answers show once you've answered.")}
    {:else if criterion.count > 0}
      {t('Members {avg} · {n} answered', {
        avg: criterion.average.toFixed(1),
        n: criterion.count,
      })}
    {/if}
  </p>
</div>
