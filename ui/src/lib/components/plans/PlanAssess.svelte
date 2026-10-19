<script lang="ts">
  import { APPROVAL_PERCENT, type VoteChoice } from '#lib/domain/governance.ts';
  import { allCriteriaRated, type PlanRating, type PlanView } from '#lib/domain/plans.ts';

  import CriterionRatingRow from './CriterionRatingRow.svelte';

  /**
   * web's plan sheet "Assess" panel: rate every criterion, then the final Yes/No. Ratings can
   * change; the final vote is final (ADR 0033), and closes once the plan is decided.
   */
  interface Props {
    plan: PlanView;
    canVote: boolean;
    onRate: (criterion: string, rating: PlanRating | null) => void;
    onVote: (choice: VoteChoice) => void;
  }

  let { plan, canVote, onRate, onVote }: Props = $props();

  // Shown at once; the next load brings the network's view.
  let local = $state<Record<string, PlanRating | null>>({});
  let localVote = $state<VoteChoice>();

  const criteria = $derived(
    plan.criteria.map((c) => (c.id in local ? { ...c, mine: local[c.id] ?? null } : c)),
  );
  const myVote = $derived(localVote ?? plan.tally.myVote);
  const rated = $derived(criteria.filter((c) => c.mine !== null).length);
  const complete = $derived(allCriteriaRated(criteria));
  const open = $derived(plan.tally.status === 'Open');
  const raters = $derived(
    Math.max(plan.tally.yes + plan.tally.no, ...criteria.map((c) => c.count)),
  );
  const percent = $derived(
    plan.tally.yes + plan.tally.no === 0
      ? 0
      : Math.round((plan.tally.yes / (plan.tally.yes + plan.tally.no)) * 100),
  );

  function rate(id: string, rating: PlanRating | null): void {
    local = { ...local, [id]: rating };
    onRate(id, rating);
  }

  function vote(choice: VoteChoice): void {
    localVote = choice;
    onVote(choice);
  }
</script>

<section class="assess-panel" data-participation-action="assess-plan">
  <div class="assess-heading">
    <h3>Assess</h3>
    <span class="approval-summary">
      {percent}% approved · {raters}
      {raters === 1 ? 'person' : 'people'} rated{#if plan.average !== null}
        · Avg {plan.average.toFixed(1)}{/if}
    </span>
  </div>
  <p class="assess-helper">
    Rate each criterion from 1 (strongly oppose) to 5 (strongly support). Community averages stay
    visible while you rate.
  </p>
  {#if !canVote}
    <p class="assess-blocked">Only members can rate this plan. Join to take part.</p>
  {/if}

  <div class="criterion-stack">
    {#each criteria as entry (entry.id)}
      <CriterionRatingRow
        criterion={entry}
        {canVote}
        onSelect={(rating) => rate(entry.id, rating)}
      />
    {/each}
  </div>
  {#if canVote}
    <p class="rating-progress">{rated}/{criteria.length} rated</p>
  {/if}

  <div class="overall-row">
    <div class="criterion-copy">
      <span class="meta-label">Final approval</span>
      <strong>Does this plan meet the need?</strong>
      <span class="threshold-note">Needs {APPROVAL_PERCENT}% yes to pass.</span>
    </div>
    <div class="overall-actions">
      {#each ['Yes', 'No'] as const as choice (choice)}
        <button
          class:selected={myVote === choice}
          class:negative={choice === 'No'}
          class="vote-chip"
          disabled={!canVote || !complete || !open || !!myVote}
          type="button"
          onclick={() => vote(choice)}
        >
          {choice}
        </button>
      {/each}
    </div>
    {#if canVote && open && !complete}
      <p class="rating-progress">Rate every criterion before the final vote.</p>
    {:else if myVote}
      <p class="rating-progress">Your vote is final once cast.</p>
    {:else if !open}
      <p class="rating-progress">This vote has closed.</p>
    {/if}
  </div>
</section>
