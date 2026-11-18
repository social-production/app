<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { APPROVAL_PERCENT, type VoteChoice } from '#lib/domain/governance.ts';
  import { allCriteriaRated, planChip, type PlanRating, type PlanView } from '#lib/domain/plans.ts';
  import { relativeCompact } from '#lib/domain/time.ts';

  import PlanAssess from './PlanAssess.svelte';
  import PlanDetails from './PlanDetails.svelte';

  import '#lib/components/styles/plans.css';

  /**
   * An opened plan (Plan, Phase and Feedback Screens, 3). It reads like a page, with the
   * result so far and "Review this plan" at the bottom. The review asks every question with
   * large targets and counts what is answered; the final Yes / No stays in view and wakes up
   * once every question is answered. The final vote is final (ADR 0033).
   */
  interface Props {
    open: boolean;
    plan: PlanView;
    authorName: string;
    /** "Production plan · makes this a project", on a proposal. */
    kindLine: string;
    canVote: boolean;
    onRate: (criterion: string, rating: PlanRating | null) => void;
    onVote: (choice: VoteChoice) => void;
    onClose: () => void;
  }

  let { open, plan, authorName, kindLine, canVote, onRate, onVote, onClose }: Props = $props();

  let reviewing = $state(false);
  // Shown at once; the next load brings the network's view.
  let local = $state<Record<string, PlanRating | null>>({});
  let localVote = $state<VoteChoice>();

  $effect(() => {
    if (!open) reviewing = false;
  });

  const criteria = $derived(
    plan.criteria.map((c) => (c.id in local ? { ...c, mine: local[c.id] ?? null } : c)),
  );
  const myVote = $derived(localVote ?? plan.tally.myVote);
  const answered = $derived(criteria.filter((c) => c.mine !== null).length);
  const complete = $derived(allCriteriaRated(criteria));
  const voteOpen = $derived(plan.tally.status === 'Open');
  const voted = $derived(plan.tally.yes + plan.tally.no);
  const percent = $derived(voted === 0 ? 0 : Math.round((plan.tally.yes / voted) * 100));
  const chip = $derived(planChip(plan));

  function rate(id: string, rating: PlanRating | null): void {
    local = { ...local, [id]: rating };
    onRate(id, rating);
  }

  function vote(choice: VoteChoice): void {
    localVote = choice;
    reviewing = false;
    onVote(choice);
  }
</script>

<OverlaySheet
  {open}
  title={reviewing ? t('Your review') : plan.plan.title}
  labelledById={`plan-sheet-${plan.id}`}
  rootClass="plan-scope"
  wide
  {onClose}
>
  {#snippet subtitle()}
    <p class="sheet-author">
      {#if reviewing}
        {plan.plan.title} · {answered} of {criteria.length} answered
      {:else}
        {kindLine} · by {authorName} · {relativeCompact(plan.createdAt)}
      {/if}
    </p>
  {/snippet}
  {#snippet headerActions()}
    {#if reviewing}
      <button class="sheet-back" type="button" onclick={() => (reviewing = false)}
        >{t('Back')}</button
      >
    {:else if chip}
      <span class="sheet-chip" class:leading={chip !== 'Rejected'}>{chip}</span>
    {/if}
  {/snippet}
  <div class="plan-sheet">
    {#if reviewing}
      <PlanAssess {criteria} {canVote} onRate={rate} />
    {:else}
      <PlanDetails plan={plan.plan} onMap={onClose} />
    {/if}
  </div>
  {#snippet footer()}
    <div class="plan-sheet-foot">
      {#if reviewing}
        <p class="plan-final">
          <strong>{t('Does this plan meet the need?')}</strong>
          {#if !voteOpen}{t('This vote has closed.')}
          {:else if !complete}{t('Answer every question first.')}
          {:else}{t('Your vote is final once cast.')}{/if}
        </p>
        <div class="plan-yes-no">
          {#each ['No', 'Yes'] as const as choice (choice)}
            <button
              class:selected={myVote === choice}
              class:yes={choice === 'Yes'}
              disabled={!canVote || !complete || !voteOpen || !!myVote}
              type="button"
              onclick={() => vote(choice)}>{choice === 'Yes' ? t('Yes, approve') : t('No')}</button
            >
          {/each}
        </div>
      {:else}
        <div class="plan-result">
          <span><strong>{percent}% yes</strong> · {voted} of {plan.tally.population} voted</span>
          <i style={`--share: ${percent}%`}></i>
          <span>needs {APPROVAL_PERCENT}%</span>
        </div>
        {#if myVote}
          <p class="plan-final">
            You voted {myVote === 'Yes' ? 'yes' : myVote.toLowerCase()}. Your vote is final once
            cast.
          </p>
        {:else if canVote && voteOpen}
          <button class="plan-review-button" type="button" onclick={() => (reviewing = true)}
            >{t('Review this plan')}</button
          >
        {/if}
      {/if}
    </div>
  {/snippet}
</OverlaySheet>
