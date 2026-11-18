<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import VoteChoiceBar from '#lib/components/governance/VoteChoiceBar.svelte';
  import VoteKicker from '#lib/components/governance/VoteKicker.svelte';
  import VoteTally from '#lib/components/governance/VoteTally.svelte';
  import type { VoteChoice } from '#lib/domain/governance.ts';
  import {
    planChip,
    type PlanCriterion,
    type PlanRating,
    type PlanView,
  } from '#lib/domain/plans.ts';
  import { relativeTime } from '#lib/domain/time.ts';

  import CriterionRatingRow from './CriterionRatingRow.svelte';
  import PlanDetails from './PlanDetails.svelte';

  import '#lib/components/styles/plans.css';

  /**
   * An opened plan as the one vote sheet (owner, 2026-11-18): the plan, the count, and
   * "Review · N questions" in the dock. The review asks one question per screen; the last
   * one's dock is No / Yes. The final vote is final (ADR 0033).
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

  /** The question on screen, or null for the plan itself. */
  let question = $state<number | null>(null);
  // Shown at once; the next load brings the network's view.
  let local = $state<Record<string, PlanRating | null>>({});
  let localVote = $state<VoteChoice>();

  $effect(() => {
    if (!open) question = null;
  });

  const criteria = $derived(
    plan.criteria.map((c) => (c.id in local ? { ...c, mine: local[c.id] ?? null } : c)),
  );
  const myVote = $derived(localVote ?? plan.tally.myVote);
  const voteOpen = $derived(plan.tally.status === 'Open');
  const chip = $derived(planChip(plan));
  const current = $derived(question === null ? null : (criteria[question] ?? null));
  const last = $derived(question !== null && question === criteria.length - 1);
  const allRated = $derived(criteria.every((c) => c.mine !== null));
  const ask = (c: PlanCriterion): string =>
    c.kind === 'Value' ? t('How well does it satisfy "{value}"?', { value: c.label }) : c.label;

  function rate(id: string, rating: PlanRating | null): void {
    local = { ...local, [id]: rating };
    onRate(id, rating);
  }

  function step(by: number): void {
    if (question !== null) question = Math.max(question + by, 0);
  }

  function vote(choice: VoteChoice): void {
    localVote = choice;
    question = null;
    onVote(choice);
  }
</script>

{#snippet back()}
  <button class="vote-back" type="button" aria-label={t('Back')} onclick={() => step(-1)}>‹</button>
{/snippet}

<OverlaySheet
  {open}
  title={current ? ask(current) : plan.plan.title}
  labelledById={`plan-sheet-${plan.id}`}
  rootClass="plan-scope vote-dock-root"
  wide
  {onClose}
>
  {#snippet kicker()}
    {#if current && question !== null}
      <VoteKicker
        label={t('Plan')}
        status={current.kind === 'Value' ? t('Shared value') : null}
        meta={t('{n} of {total}', { n: question + 1, total: criteria.length })}
      />
    {:else}
      <VoteKicker
        label={kindLine}
        status={chip ? t(chip) : null}
        meta={`${authorName} · ${relativeTime(plan.createdAt)}`}
      />
    {/if}
  {/snippet}
  <div class="plan-sheet vote-sheet">
    {#if current}
      <CriterionRatingRow
        criterion={current}
        {canVote}
        onSelect={(rating) => rate(current.id, rating)}
      />
      {#if last}<p class="outcome">{t('Does this plan meet the need? Your vote is final.')}</p>{/if}
    {:else}
      <PlanDetails plan={plan.plan} onMap={onClose} />
      <VoteTally
        yes={plan.tally.yes}
        no={plan.tally.no}
        population={plan.tally.population}
        failed={plan.tally.status === 'Rejected'}
      />
    {/if}
  </div>
  {#snippet footer()}
    {#if current && question !== null}
      {#if last}
        <VoteChoiceBar docked mine={null} disabled={!allRated} onChoose={vote}>
          {#snippet lead()}{@render back()}{/snippet}
        </VoteChoiceBar>
      {:else}
        <div class="sheet-actions">
          {#if question > 0}{@render back()}{/if}
          <button
            class="sheet-submit"
            type="button"
            disabled={!current.mine}
            onclick={() => step(1)}>{t('Next')}</button
          >
        </div>
      {/if}
    {:else if myVote}
      <VoteChoiceBar docked mine={myVote} onChoose={() => undefined} />
    {:else if canVote && voteOpen && criteria.length === 0}
      <VoteChoiceBar docked mine={null} onChoose={vote} />
    {:else if canVote && voteOpen}
      <div class="sheet-actions">
        <button class="sheet-submit" type="button" onclick={() => (question = 0)}
          >{t('Review · {n} questions', { n: criteria.length })}</button
        >
      </div>
    {/if}
  {/snippet}
</OverlaySheet>
