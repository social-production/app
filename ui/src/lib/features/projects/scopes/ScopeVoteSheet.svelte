<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import type { VoteChoice } from '#lib/domain/governance.ts';

  import {
    describeDecision,
    type ScopeDecision,
    type ScopeGovernance,
  } from './scopeGovernance.svelte.ts';

  import '#lib/components/styles/scopes.css';

  /**
   * web's ScopeVoteSheet: the open decisions you haven't voted on, one at a time; voting moves
   * to the next. Votes are final here (ADR 0033), so there is no changing one afterwards.
   */
  interface Props {
    open: boolean;
    gov: ScopeGovernance;
    memberName: (agent: string) => string;
    onClose: () => void;
  }

  let { open, gov, memberName, onClose }: Props = $props();

  let index = $state(0);
  let voted = $state<VoteChoice | null>(null);
  let notice = $state('');

  $effect(() => {
    if (!open) return;
    index = 0;
    voted = null;
    notice = '';
  });

  // While the "You voted…" line shows, keep the decision just voted on in view.
  let shown = $state<ScopeDecision | null>(null);
  const votable = $derived(gov.votable);
  const current = $derived(shown ?? votable[index] ?? null);

  const formatWhen = (micros: number): string =>
    new Date(micros / 1000).toLocaleString(undefined, {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    });

  async function cast(item: ScopeDecision, choice: VoteChoice): Promise<void> {
    shown = item;
    if (!(await gov.vote(item.id, choice))) {
      shown = null;
      return;
    }
    voted = choice;
    const stillOpen = gov.open.some((p) => p.id === item.id);
    notice = stillOpen ? 'Vote recorded' : 'Vote recorded. That decision is now settled.';
    // Leave "You voted…" on screen for a moment before moving on.
    await new Promise((resolve) => setTimeout(resolve, 1200));
    shown = null;
    voted = null;
    index = 0;
    if (gov.votable.length === 0) onClose();
  }
</script>

<OverlaySheet {open} title="Vote" labelledById="scope-vote-title" rootClass="scope-vote" {onClose}>
  <div class="vote-body">
    {#if gov.message}<p class="meta">{gov.message}</p>{/if}
    {#if current}
      <p class="what">{describeDecision(current, memberName)}</p>
      <p class="meta">Proposed by {current.proposer}</p>
      {#if 'Edit' in current.kind && current.kind.Edit.description !== null}
        <blockquote>{current.kind.Edit.description}</blockquote>
      {/if}
      <p class="meta">
        {current.yes} yes · {current.no} no · {current.required} votes needed
      </p>
      <p class="meta">Proposed {formatWhen(current.createdAt)}</p>
      {#if voted}
        <p class="voted">You voted {voted}. {notice}</p>
      {/if}
      <div class="choices">
        <button
          class="no"
          type="button"
          class:chosen={voted === 'No'}
          aria-pressed={voted === 'No'}
          disabled={gov.busy || voted !== null}
          onclick={() => current && void cast(current, 'No')}>No</button
        >
        <button
          class="yes"
          type="button"
          class:chosen={voted === 'Yes'}
          aria-pressed={voted === 'Yes'}
          disabled={gov.busy || voted !== null}
          onclick={() => current && void cast(current, 'Yes')}>Yes</button
        >
      </div>
      {#if votable.length > 1 && !shown}
        <div class="nav">
          <span class="meta">{index + 1} of {votable.length} · {votable.length} not voted yet</span>
          <button class="next" type="button" onclick={() => (index = (index + 1) % votable.length)}>
            Next
          </button>
        </div>
      {/if}
    {:else if !gov.loaded}
      <p class="meta">Loading…</p>
    {:else}
      <p class="meta">No open votes. You're all caught up.</p>
    {/if}
  </div>
</OverlaySheet>
