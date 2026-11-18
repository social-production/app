<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { proposalLabel, type Proposal, type VoteChoice } from '#lib/domain/governance.ts';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import {
    historyIcon,
    requirementLabel,
    statusLabel,
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
  const approval = $derived(
    entry.yes + entry.no > 0 ? Math.round((entry.yes / (entry.yes + entry.no)) * 100) : 0,
  );
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
    <!-- Polish Pass mockups: a decided one is a row with its tag; an open one is a vote card. -->
    <button
      class="history-toggle"
      type="button"
      aria-label={`${entry.kindLabel}, ${statusLabel(entry.status)}`}
      onclick={open}
    >
      <span class="history-icon"><ToolbarIcon name={historyIcon(entry.kindLabel)} /></span>
      <span class="history-copy">
        <span class="history-kicker">{entry.kindLabel}</span>
        <span class="history-meta"
          >{names[entry.author] ?? '…'} · {relativeTime(entry.createdAt)} · {entry.yes} yes, {entry.no}
          no</span
        >
      </span>
      {#if entry.status !== 'open'}
        <span class="history-tag">{entry.status === 'approved' ? t('Passed') : t('Failed')}</span>
      {/if}
    </button>
    {#if entry.status === 'open'}
      <div class="history-progress">
        <div class="history-bar"><i style={`width: ${Math.max(approval, 3)}%`}></i></div>
        <span class="history-requirement">{requirementLabel(entry)}</span>
      </div>
      {#if entry.proposal}
        {@const proposal = entry.proposal}
        <VoteChoiceBar mine={proposal.myVote} onChoose={(c) => cast(proposal, c)} />
      {:else if entry.plan}
        <button class="history-review" type="button" onclick={open}>{t('Review this plan')}</button>
      {/if}
    {/if}
  </article>
</div>

<OverlaySheet
  open={sheet !== null}
  title={sheet ? proposalLabel(sheet.kind, phaseLabel) : t('Vote')}
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
