<script lang="ts">
  import { APPROVAL_PERCENT, type VoteChoice } from '#lib/domain/governance.ts';
  import {
    resultNote,
    voteRequirement,
    type LinkHalf,
    type LinkRequest,
    type LinkSide,
  } from '#lib/domain/links.ts';

  import '#lib/components/styles/links.css';

  /**
   * One open link vote as a single card (Polish Pass mockups, 2026-11-16): what is asked, who
   * asked, a bar per project, and Approve / Oppose on the side you belong to. Votes are final
   * (ADR 0033), so the chosen button stays marked and both disable.
   */
  interface Props {
    request: LinkRequest;
    proposer: string;
    pending?: boolean;
    yesLabel?: string;
    noLabel?: string;
    danger?: boolean;
    /** Class names the card answers to (a request in "Votes open" is a `link-card`). */
    cardClass?: string;
    domId?: string;
    onVote?: (side: LinkSide, choice: VoteChoice) => void;
  }

  let {
    request,
    proposer,
    pending = false,
    yesLabel = 'Approve',
    noLabel = 'Oppose',
    danger = false,
    cardClass = '',
    domId,
    onVote,
  }: Props = $props();

  const title = $derived(
    request.sever
      ? `End the link with ${request.target.title}`
      : `Link with ${request.target.title}`,
  );
  const percent = (h: LinkHalf | null): number =>
    h && h.yes + h.no > 0 ? Math.round((h.yes / (h.yes + h.no)) * 100) : 0;
  // The mockup's "1 of 1 · 100%": who has voted of the members, and the share saying yes.
  const tally = (h: LinkHalf | null): string =>
    h ? `${h.yes + h.no} of ${h.population} · ${percent(h)}%` : 'Not received yet';
  // One Approve / Oppose pair for the side the viewer belongs to (Polish Pass mockups §6).
  const mine = $derived(
    [request.thisSide, request.otherSide].find((s) => s.canVote || s.half?.myVote),
  );
  const open = $derived(request.status === 'Open');
</script>

<article id={domId} class={`link-vote ${cardClass}`}>
  <strong class="link-vote-title">{title}</strong>
  <span class="link-vote-meta">Proposed by {proposer}</span>
  {#each [request.thisSide, request.otherSide] as side (side.projectId)}
    {@const half = side.half}
    <div class="link-vote-side">
      <div class="link-vote-line">
        <b>{side === request.thisSide ? 'This project' : side.projectTitle}</b>
        <span>{tally(half)}</span>
      </div>
      <div class="link-vote-bar" class:done={half?.status === 'Approved'}>
        <i style={`width: ${Math.max(percent(half), 3)}%`}></i>
      </div>
      {#if !open || half?.status !== 'Open'}
        <span class="link-vote-note"
          >{half?.status === 'Open' ? voteRequirement(half) : resultNote(half)}</span
        >
      {/if}
    </div>
  {/each}
  {#if open}<span class="link-vote-note">Each side needs {APPROVAL_PERCENT}% of its members.</span
    >{/if}
  {#if onVote && mine}
    {@const side = mine}
    {@const voted = side.half?.myVote ?? null}
    <div class="vote-buttons">
      <button
        class:active={voted === 'Yes'}
        class:danger
        class="vote-button yes"
        disabled={pending || !side.canVote}
        type="button"
        onclick={() => onVote(side, 'Yes')}>{yesLabel}</button
      >
      <button
        class:active={voted === 'No'}
        class="vote-button"
        disabled={pending || !side.canVote}
        type="button"
        onclick={() => onVote(side, 'No')}>{noLabel}</button
      >
    </div>
  {/if}
</article>
