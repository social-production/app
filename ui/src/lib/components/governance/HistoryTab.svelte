<script lang="ts">
  import type { Proposal, VoteChoice } from '#lib/domain/governance.ts';
  import { historyEntries, stageBefore, type HistoryEntry } from '#lib/domain/history.ts';
  import type { PlanView } from '#lib/domain/plans.ts';
  import type { Charter } from '#lib/domain/projects.ts';

  import DecisionHistoryCard from './DecisionHistoryCard.svelte';

  import '#lib/components/styles/history.css';

  /**
   * web's ProjectHistoryTab / EventHistoryTab: `DecisionHistoryList` (Active Decisions, then
   * Resolutions) of `DecisionHistoryCard`s. A card opens the decision's vote sheet (open ones
   * can still be voted, as on web); a plan opens its own sheet in its phase. A software
   * project's decisions (ADR 0086) come in `extra`.
   */
  interface Props {
    proposals: Proposal[];
    /** `undefined` while the plans load. */
    plans: PlanView[] | undefined;
    names: Record<string, string>;
    charter: Charter;
    stage: string;
    phaseLabel: (id: string) => string;
    onVote: (proposal: Proposal, choice: VoteChoice) => Promise<void>;
    onOpenPlan: (plan: PlanView) => void;
    extra?: HistoryEntry[];
  }

  let {
    proposals,
    plans,
    names,
    charter,
    stage,
    phaseLabel,
    onVote,
    onOpenPlan,
    extra = [],
  }: Props = $props();

  const entries = $derived(
    [...historyEntries(proposals, plans ?? [], phaseLabel), ...extra].sort(
      (a, b) => b.createdAt - a.createdAt,
    ),
  );
  const loading = $derived(plans === undefined);
  // web's wording differs between projects and events only in that word.
  const noun = $derived(charter.mode === 'event' ? 'event' : 'project');
  const active = $derived(entries.filter((e) => e.status === 'open'));
  const resolved = $derived(entries.filter((e) => e.status !== 'open'));

  const stageOf = (p: Proposal): string =>
    p.status === 'Open' ? stage : phaseLabel(stageBefore(p, proposals, ''));
</script>

{#snippet card(entry: HistoryEntry)}
  <DecisionHistoryCard
    {entry}
    {names}
    {charter}
    {stageOf}
    {phaseLabel}
    {onVote}
    onOpenPlan={(e) => e.plan && onOpenPlan(e.plan)}
  />
{/snippet}

<section class="history-shell">
  <div class="section-copy">
    <h2>History</h2>
    <p>
      Open, approved, and rejected {noun} decisions stay here in one timeline. Open decisions can still
      be voted from this tab.
    </p>
  </div>

  <section class="history-group">
    <div class="group-head">
      <h3>Active Decisions</h3>
      <span>{active.length}</span>
    </div>
    {#if active.length === 0}
      <div class="empty-card"><p>No active decisions.</p></div>
    {:else}
      <div class:scrollable={active.length > 5} class="history-list">
        {#each active as entry (entry.id)}{@render card(entry)}{/each}
      </div>
    {/if}
  </section>

  <section class="history-group">
    <div class="group-head">
      <h3>Resolutions</h3>
      <span>{resolved.length}</span>
    </div>
    {#if resolved.length === 0}
      <div class="empty-card">
        <p>
          {#if entries.length > 0}No resolutions yet.
          {:else if loading}Loading {noun} history…
          {:else}No {noun} decision history yet.{/if}
        </p>
      </div>
    {:else}
      <div class:scrollable={resolved.length > 5} class="history-list">
        {#each resolved as entry (entry.id)}{@render card(entry)}{/each}
      </div>
    {/if}
  </section>
</section>
