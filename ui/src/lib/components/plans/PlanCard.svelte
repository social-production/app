<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import type { VoteChoice } from '#lib/domain/governance.ts';
  import {
    allCriteriaRated,
    approvalLine,
    planChip,
    type PlanRating,
    type PlanView,
  } from '#lib/domain/plans.ts';
  import { relativeCompact } from '#lib/domain/time.ts';

  import PlanAssess from './PlanAssess.svelte';
  import PlanDetails from './PlanDetails.svelte';

  import '#lib/components/styles/plans.css';

  /**
   * web's CollapsiblePlanCard: a compact card (title, chips, stages, approval) that opens the
   * plan sheet with its details and the Assess panel. Plans can't be edited once posted.
   */
  interface Props {
    plan: PlanView;
    authorName: string;
    canVote: boolean;
    onRate: (criterion: string, rating: PlanRating | null) => void;
    onVote: (choice: VoteChoice) => void;
  }

  let { plan, authorName, canVote, onRate, onVote }: Props = $props();

  let open = $state(false);

  const chip = $derived(planChip(plan));
  const leading = $derived(plan.leader === 'Leading');
  const tied = $derived(plan.leader === 'Tied');
  const voteOpen = $derived(plan.tally.status === 'Open' && !plan.tally.myVote);
  const actionChip = $derived(
    !canVote ? null : !allCriteriaRated(plan.criteria) ? 'Assess' : voteOpen ? 'Vote' : null,
  );
  const sheetId = $derived(`plan-sheet-${plan.id}`);
</script>

<div class="plan-scope" style="display: contents">
  <button
    id={`vote-card-plan-${plan.id}`}
    class="surface-card plan-card"
    class:leading
    class:tied
    class:open
    class:needs-action={!!actionChip}
    type="button"
    data-participation-action={actionChip ? 'assess-plan' : undefined}
    onclick={() => (open = true)}
  >
    <span class="plan-header">
      <strong class="plan-title">{plan.plan.title}</strong>
      <span class="plan-chips">
        {#if actionChip}<span class="assess-chip">{actionChip}</span>{/if}
        {#if chip}
          <span class="phase-badge" class:complete={leading} class:tied-badge={tied}>{chip}</span>
        {/if}
      </span>
    </span>
    <span class="plan-lede collapsed-copy">{plan.plan.description}</span>
    <span class="plan-stage-list">
      {#each plan.plan.stages as stage, index (index)}
        <span>{index + 1}. {stage.title}</span>
      {/each}
    </span>
    <span class="plan-footer-meta">
      <span>
        {approvalLine(plan.tally)}{plan.average !== null ? ` · Avg ${plan.average.toFixed(1)}` : ''}
      </span>
      <span>{authorName} · {relativeCompact(plan.createdAt)}</span>
    </span>
  </button>
</div>

<OverlaySheet
  {open}
  title={plan.plan.title}
  labelledById={sheetId}
  rootClass="plan-scope"
  wide
  onClose={() => (open = false)}
>
  {#snippet subtitle()}
    {#if chip}<p class="sheet-kicker">{chip}</p>{/if}
    <p class="sheet-author">{authorName} · {relativeCompact(plan.createdAt)}</p>
  {/snippet}
  <div class="plan-sheet">
    <PlanDetails plan={plan.plan} />
    <PlanAssess {plan} {canVote} {onRate} {onVote} />
  </div>
</OverlaySheet>
