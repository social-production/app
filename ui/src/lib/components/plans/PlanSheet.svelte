<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import VoteChoiceBar from '#lib/components/governance/VoteChoiceBar.svelte';
  import VoteKicker from '#lib/components/governance/VoteKicker.svelte';
  import VoteTally from '#lib/components/governance/VoteTally.svelte';
  import type { VoteChoice } from '#lib/domain/governance.ts';
  import {
    criterionQuestion,
    criterionTag,
    planChip,
    type PlanRating,
    type PlanView,
  } from '#lib/domain/plans.ts';
  import { relativeTime } from '#lib/domain/time.ts';

  import PlanDetails from './PlanDetails.svelte';
  import PlanQuestion from './PlanQuestion.svelte';
  import PlanReviewSummary from './PlanReviewSummary.svelte';

  import '#lib/components/styles/plans.css';

  /**
   * An opened plan as the one vote sheet (owner, 2026-11-18; Plan review mockup, 2026-11-25):
   * the plan and its count; then one question per screen, answered in words with one tap that
   * moves on; then your answers on one page with the final No / Yes (ADR 0033).
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

  /** A question's index, your answers, or null for the plan itself. */
  let screen = $state<number | 'answers' | null>(null);
  // Changing one answer from the answers page goes back there.
  let fromAnswers = $state(false);
  // Shown at once; the next load brings the network's view.
  let local = $state<Record<string, PlanRating | null>>({});
  let localVote = $state<VoteChoice>();

  $effect(() => {
    if (!open) screen = null;
  });

  const criteria = $derived(
    plan.criteria.map((c) => (c.id in local ? { ...c, mine: local[c.id] ?? null } : c)),
  );
  const myVote = $derived(localVote ?? plan.tally.myVote);
  const voteOpen = $derived(plan.tally.status === 'Open');
  const chip = $derived(planChip(plan));
  const current = $derived(typeof screen === 'number' ? (criteria[screen] ?? null) : null);
  const allRated = $derived(criteria.every((c) => c.mine !== null));
  const total = $derived(criteria.length);
  const done = $derived(screen === 'answers' ? total : typeof screen === 'number' ? screen : 0);

  function answer(index: number, rating: PlanRating): void {
    const c = criteria[index];
    if (c && c.mine !== rating) {
      local = { ...local, [c.id]: rating };
      onRate(c.id, rating);
    }
    // A moment to see the choice land, then on.
    setTimeout(() => {
      screen = fromAnswers || index + 1 >= total ? 'answers' : index + 1;
      fromAnswers = false;
    }, 220);
  }

  function back(): void {
    if (screen === 'answers') screen = total - 1;
    else if (typeof screen === 'number') screen = screen > 0 ? screen - 1 : null;
  }

  function begin(): void {
    fromAnswers = false;
    screen = allRated ? 'answers' : criteria.findIndex((c) => c.mine === null);
  }

  function vote(choice: VoteChoice): void {
    localVote = choice;
    screen = null;
    onVote(choice);
  }
</script>

{#snippet backButton()}
  <button class="vote-back" type="button" aria-label={t('Back')} onclick={back}>‹</button>
{/snippet}

<OverlaySheet
  {open}
  title={current
    ? criterionQuestion(current)
    : screen === 'answers'
      ? t('Your review')
      : plan.plan.title}
  labelledById={`plan-sheet-${plan.id}`}
  rootClass="plan-scope vote-dock-root"
  wide
  {onClose}
>
  {#snippet kicker()}
    {#if screen !== null}
      <div class="plan-kicker">
        <div class="plan-progress" aria-hidden="true">
          {#each criteria as c, i (c.id)}<i class:on={i < done || i === screen}></i>{/each}
        </div>
        {#if current && typeof screen === 'number'}
          <div class="plan-q-top">
            <span>{t('Question {n} of {total}', { n: screen + 1, total })}</span>
            <span class="plan-q-tag">{criterionTag(current)}</span>
          </div>
        {/if}
      </div>
    {:else}
      <VoteKicker
        label={kindLine}
        status={chip ? t(chip) : null}
        meta={`${authorName} · ${relativeTime(plan.createdAt)}`}
      />
    {/if}
  {/snippet}
  <div class="plan-sheet vote-sheet">
    {#if current && typeof screen === 'number'}
      {@const index = screen}
      <PlanQuestion
        criterion={current}
        planTitle={plan.plan.title}
        {canVote}
        onAnswer={(rating) => answer(index, rating)}
      />
    {:else if screen === 'answers'}
      <PlanReviewSummary
        {criteria}
        onEdit={(i) => {
          fromAnswers = true;
          screen = i;
        }}
      />
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
    {#if typeof screen === 'number'}
      <div class="sheet-actions">{@render backButton()}</div>
    {:else if screen === 'answers'}
      <VoteChoiceBar docked mine={myVote ?? null} disabled={!allRated} onChoose={vote}>
        {#snippet lead()}{@render backButton()}{/snippet}
      </VoteChoiceBar>
    {:else if myVote}
      <VoteChoiceBar docked mine={myVote} onChoose={() => undefined} />
    {:else if canVote && voteOpen && total === 0}
      <VoteChoiceBar docked mine={null} onChoose={vote} />
    {:else if canVote && voteOpen}
      <div class="sheet-actions">
        <button class="sheet-submit plan-begin" type="button" onclick={begin}
          >{t('Review and vote')}<small
            >{total === 1 ? t('1 quick question') : t('{n} quick questions', { n: total })}</small
          ></button
        >
      </div>
    {/if}
  {/snippet}
</OverlaySheet>
