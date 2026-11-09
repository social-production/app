<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { proposalLabel, type Proposal, type VoteChoice } from '#lib/domain/governance.ts';
  import {
    requirementLabel,
    statusLabel,
    voteSummaryLabel,
    type HistoryEntry,
  } from '#lib/domain/history.ts';
  import type { Charter } from '#lib/domain/projects.ts';
  import { relativeTime } from '#lib/domain/time.ts';

  import VoteChoiceBar from './VoteChoiceBar.svelte';
  import VoteDecisionBody from './VoteDecisionBody.svelte';

  import '#lib/components/styles/governance.css';
  import '#lib/components/styles/history.css';

  /**
   * web's DecisionHistoryCard: a decision's status, requirement, tally and author; it opens the
   * decision's vote sheet (open ones can still be voted). A plan opens its own sheet instead.
   */
  interface Props {
    entry: HistoryEntry;
    names: Record<string, string>;
    charter: Charter;
    /** The stage a decision's sheet says it moves from. */
    stageOf: (p: Proposal) => string;
    phaseLabel: (id: string) => string;
    onVote: (proposal: Proposal, choice: VoteChoice) => Promise<void>;
    onOpenPlan?: (entry: HistoryEntry) => void;
  }

  let { entry, names, charter, stageOf, phaseLabel, onVote, onOpenPlan }: Props = $props();

  let showing = $state(false);
  // The sheet reads the live entry, so a vote shows in it once the page re-reads.
  const sheet = $derived(showing ? entry.proposal : null);

  function open(): void {
    if (entry.plan) onOpenPlan?.(entry);
    else showing = true;
  }

  function cast(p: Proposal, choice: VoteChoice): void {
    showing = false;
    void onVote(p, choice);
  }
</script>

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
      onclick={open}
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

<OverlaySheet
  open={sheet !== null}
  title={sheet ? proposalLabel(sheet.kind, phaseLabel) : 'Vote'}
  labelledById={`history-vote-${entry.id}`}
  rootClass="vote-dock-root"
  onClose={() => (showing = false)}
>
  {#if sheet}
    <div class="vote-list">
      <article class="vote-block">
        <VoteDecisionBody
          proposal={sheet}
          authorName={names[sheet.author] ?? '…'}
          {charter}
          stage={stageOf(sheet)}
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
