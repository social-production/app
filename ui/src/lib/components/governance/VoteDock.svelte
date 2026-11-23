<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import type { Proposal, VoteChoice } from '#lib/domain/governance.ts';
  import { relativeTime } from '#lib/domain/time.ts';
  import { voteHeadline, voteKindLabel } from '#lib/domain/voteSheet.ts';
  import type { Charter } from '#lib/domain/projects.ts';

  import VoteChoiceBar from './VoteChoiceBar.svelte';
  import VoteDecisionBody from './VoteDecisionBody.svelte';
  import VoteKicker from './VoteKicker.svelte';

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
    /** A proposal a link asks to open (web's `?open=vote&voteTarget=…`). */
    reveal?: string | null;
    /** The revealed sheet closed: the link is dropped, so it doesn't reopen. */
    onRevealDone?: (() => void) | undefined;
    /** The viewer isn't a Full member yet, so their vote doesn't decide (ADR 0033). */
    newVoter?: boolean;
  }

  let {
    proposals,
    names,
    charter,
    stage,
    phaseLabel,
    onVote,
    reveal = null,
    onRevealDone,
    newVoter = false,
  }: Props = $props();

  let open = $state(false);
  let index = $state(0);
  let voted = $state<string[]>([]);

  const current = $derived(proposals[index] ?? proposals[0]);
  const remaining = $derived(proposals.filter((p) => !voted.includes(p.id) && !p.myVote).length);

  let revealed = $state<string | null>(null);

  // web's VoteDockControl reveal: open the sheet on the linked proposal once it has loaded.
  $effect(() => {
    if (!reveal || revealed === reveal) return;
    const at = proposals.findIndex((p) => p.id === reveal);
    if (at < 0) return;
    voted = [];
    index = at;
    open = true;
    revealed = reveal;
  });

  function close(): void {
    open = false;
    if (revealed) onRevealDone?.();
  }

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
    else close();
    void onVote(p, choice);
  }
</script>

{#if proposals.length > 0}
  <div class="vote-dock-root" style="display: contents">
    <div class="vote-dock">
      <RoundPlusButton
        standout
        plus={false}
        label={t('Vote')}
        ariaLabel={t('Vote')}
        onclick={openVotes}
      />
    </div>
  </div>
{/if}

<OverlaySheet
  {open}
  title={current ? voteHeadline(current.kind, phaseLabel) : t('Vote')}
  labelledById="project-votes"
  rootClass="vote-dock-root"
  onClose={close}
>
  {#snippet kicker()}
    {#if current}
      <VoteKicker
        label={voteKindLabel(current.kind)}
        meta={`${names[current.author] ?? '…'} · ${relativeTime(current.createdAt)}`}
      />
    {/if}
  {/snippet}
  {#if current}
    <div class="vote-list">
      <article class="vote-block">
        <VoteDecisionBody proposal={current} {charter} {stage} {phaseLabel} {newVoter} />
      </article>
    </div>
  {/if}
  {#snippet footer()}
    {#if current}
      <VoteChoiceBar docked mine={current.myVote} onChoose={(c) => cast(current, c)}>
        {#snippet lead()}
          {#if proposals.length > 1}
            <button
              class="vote-next"
              type="button"
              onclick={() => (index = (index + 1) % proposals.length)}
            >
              <span>{t('Next')}</span>
              <span class="vote-progress">{remaining}</span>
            </button>
          {/if}
        {/snippet}
      </VoteChoiceBar>
    {/if}
  {/snippet}
</OverlaySheet>
