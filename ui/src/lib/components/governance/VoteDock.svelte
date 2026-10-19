<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import { proposalLabel, type Proposal, type VoteChoice } from '#lib/domain/governance.ts';
  import type { Charter } from '#lib/domain/projects.ts';

  import VoteChoiceBar from './VoteChoiceBar.svelte';
  import VoteDecisionBody from './VoteDecisionBody.svelte';

  import '#lib/components/styles/governance.css';

  /**
   * web's VoteDockControl: a "Vote" button in the action dock opening a sheet that walks
   * through open proposals; voting moves on to the next one until all are done.
   */
  interface Props {
    proposals: Proposal[];
    names: Record<string, string>;
    charter: Charter;
    stage: string;
    phaseLabel: (id: string) => string;
    onVote: (proposal: Proposal, choice: VoteChoice) => Promise<void>;
  }

  let { proposals, names, charter, stage, phaseLabel, onVote }: Props = $props();

  let open = $state(false);
  let index = $state(0);
  let voted = $state<string[]>([]);

  const current = $derived(proposals[index] ?? proposals[0]);
  const remaining = $derived(proposals.filter((p) => !voted.includes(p.id) && !p.myVote).length);

  function openVotes(): void {
    voted = [];
    const firstUnvoted = proposals.findIndex((p) => !p.myVote);
    index = Math.max(firstUnvoted, 0);
    open = true;
  }

  function cast(p: Proposal, choice: VoteChoice): void {
    voted = [...voted, p.id];
    const next = proposals.findIndex((q) => !q.myVote && !voted.includes(q.id));
    if (next >= 0) index = next;
    else open = false;
    void onVote(p, choice);
  }
</script>

{#if proposals.length > 0}
  <div class="vote-dock-root" style="display: contents">
    <div class="vote-dock">
      <RoundPlusButton standout label="Vote" ariaLabel="Vote" onclick={openVotes} />
    </div>
  </div>
{/if}

<OverlaySheet
  {open}
  title={current ? proposalLabel(current.kind, phaseLabel) : 'Vote'}
  labelledById="project-votes"
  rootClass="vote-dock-root"
  onClose={() => (open = false)}
>
  {#if current}
    <div class="vote-list">
      <article class="vote-block">
        <VoteDecisionBody
          proposal={current}
          authorName={names[current.author] ?? '…'}
          {charter}
          {stage}
          {phaseLabel}
        />
      </article>
    </div>
  {/if}
  {#snippet footer()}
    {#if current}
      <div class="vote-footer">
        {#if proposals.length > 1}
          <button
            class="vote-next"
            type="button"
            onclick={() => (index = (index + 1) % proposals.length)}
          >
            <span>Next</span>
            <span class="vote-progress">· {remaining} left</span>
            <svg aria-hidden="true" viewBox="0 0 24 24"
              ><path
                d="M9 6.5 14.5 12 9 17.5"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              /></svg
            >
          </button>
        {/if}
        <VoteChoiceBar mine={current.myVote} onChoose={(c) => cast(current, c)} />
      </div>
    {/if}
  {/snippet}
</OverlaySheet>
