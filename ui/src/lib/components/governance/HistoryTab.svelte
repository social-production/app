<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { proposalLabel, type Proposal, type VoteChoice } from '#lib/domain/governance.ts';
  import {
    historyEntries,
    requirementLabel,
    statusLabel,
    voteSummaryLabel,
    type HistoryEntry,
  } from '#lib/domain/history.ts';
  import type { PlanView } from '#lib/domain/plans.ts';
  import type { Charter } from '#lib/domain/projects.ts';
  import { relativeTime } from '#lib/domain/time.ts';

  import VoteChoiceBar from './VoteChoiceBar.svelte';
  import VoteDecisionBody from './VoteDecisionBody.svelte';

  import '#lib/components/styles/governance.css';
  import '#lib/components/styles/history.css';

  /**
   * web's ProjectHistoryTab / EventHistoryTab: `DecisionHistoryList` (Active Decisions, then
   * Resolutions) of `DecisionHistoryCard`s. A card opens the decision's vote sheet (open ones
   * can still be voted, as on web); a plan opens its own sheet in its phase.
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
  }

  let { proposals, plans, names, charter, stage, phaseLabel, onVote, onOpenPlan }: Props = $props();

  const entries = $derived(historyEntries(proposals, plans ?? [], phaseLabel));
  const loading = $derived(plans === undefined);
  // web's wording differs between projects and events only in that word.
  const noun = $derived(charter.mode === 'event' ? 'event' : 'project');
  let showing = $state<string | null>(null);
  const active = $derived(entries.filter((e) => e.status === 'open'));
  const resolved = $derived(entries.filter((e) => e.status !== 'open'));
  // The sheet reads the live entry, so a vote shows in it once the page re-reads.
  const sheet = $derived(entries.find((e) => e.id === showing)?.proposal ?? null);

  function open(entry: HistoryEntry): void {
    if (entry.plan) onOpenPlan(entry.plan);
    else showing = entry.id;
  }

  function cast(p: Proposal, choice: VoteChoice): void {
    showing = null;
    void onVote(p, choice);
  }
</script>

{#snippet card(entry: HistoryEntry)}
  <div class="history-rail-card">
    <article
      id={`decision-${entry.id}`}
      class="history-card"
      class:open={entry.status === 'open'}
      class:approved={entry.status === 'approved'}
      class:rejected={entry.status === 'rejected'}
    >
      <button
        class="history-toggle"
        type="button"
        aria-label={`${entry.kindLabel}, ${statusLabel(entry.status)}`}
        onclick={() => open(entry)}
      >
        <div class="history-status-row">
          <span class="history-kicker">{entry.kindLabel}</span>
          <span class="history-requirement">{requirementLabel(entry)}</span>
        </div>
        <div class="history-meta-row">
          <span class="history-meta-left">{voteSummaryLabel(entry)}</span>
          <span class="history-meta-right"
            >{names[entry.author] ?? '…'} · {relativeTime(entry.createdAt)}</span
          >
        </div>
      </button>
    </article>
  </div>
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

<OverlaySheet
  open={sheet !== null}
  title={sheet ? proposalLabel(sheet.kind, phaseLabel) : 'Vote'}
  labelledById="history-vote"
  rootClass="vote-dock-root"
  onClose={() => (showing = null)}
>
  {#if sheet}
    <div class="vote-list">
      <article class="vote-block">
        <VoteDecisionBody
          proposal={sheet}
          authorName={names[sheet.author] ?? '…'}
          {charter}
          {stage}
          {phaseLabel}
        />
      </article>
    </div>
  {/if}
  {#snippet footer()}
    {#if sheet && sheet.status === 'Open'}
      <div class="vote-footer">
        <VoteChoiceBar mine={sheet.myVote} onChoose={(c) => cast(sheet, c)} />
      </div>
    {/if}
  {/snippet}
</OverlaySheet>
