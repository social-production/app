<script lang="ts">
  import { APPROVAL_PERCENT, type VoteChoice } from '#lib/domain/governance.ts';

  import { describeDecision, type ScopeDecision } from './scopeGovernance.svelte.ts';

  /**
   * One scope decision as a card (Channel Pictures mockups §3): what it changes with its tag,
   * the new text, a bar that fills towards the votes needed, one line with the count and who
   * proposed it, and No / Yes while it is open and yours to vote on.
   */
  interface Props {
    item: ScopeDecision;
    memberName: (agent: string) => string;
    /** Shows No / Yes; votes are final (ADR 0033). */
    onVote?: ((choice: VoteChoice) => void) | undefined;
    busy?: boolean;
    /** The choice just made, kept marked while the sheet moves on. */
    chosen?: VoteChoice | null;
  }

  let { item, memberName, onVote, busy = false, chosen = null }: Props = $props();

  const open = $derived(item.status === 'Open');
  const share = $derived(
    item.yes + item.no > 0 ? Math.round((item.yes / (item.yes + item.no)) * 100) : 0,
  );
  const fill = $derived(
    open ? Math.min(100, Math.round((item.yes / Math.max(item.required, 1)) * 100)) : share,
  );
  const tag = $derived(
    open ? 'Open' : `${item.status === 'Passed' ? 'Passed' : 'Rejected'} · ${share}% yes`,
  );
  const text = $derived('Edit' in item.kind ? item.kind.Edit.description : null);
</script>

<article
  class="decision-card"
  class:open
  class:approved={item.status === 'Passed'}
  class:rejected={!open && item.status !== 'Passed'}
>
  <div class="decision-top">
    <strong>{describeDecision(item, memberName)}</strong>
    <span class="decision-tag">{tag}</span>
  </div>
  {#if text}<p class="decision-text">"{text}"</p>{/if}
  <div class="decision-bar" aria-hidden="true"><i style={`width: ${Math.max(fill, 3)}%`}></i></div>
  <p class="decision-line">
    {item.yes} yes · {item.no} no · {item.required} votes needed, {APPROVAL_PERCENT}% yes · by {item.proposer}
  </p>
  {#if onVote}
    <div class="decision-choices">
      <button
        class="no"
        type="button"
        class:chosen={chosen === 'No'}
        aria-pressed={chosen === 'No'}
        disabled={busy || chosen !== null}
        onclick={() => onVote('No')}>No</button
      >
      <button
        class="yes"
        type="button"
        class:chosen={chosen === 'Yes'}
        aria-pressed={chosen === 'Yes'}
        disabled={busy || chosen !== null}
        onclick={() => onVote('Yes')}>Yes</button
      >
    </div>
  {/if}
</article>
