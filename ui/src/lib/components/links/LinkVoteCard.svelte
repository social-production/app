<script lang="ts">
  import type { VoteChoice } from '#lib/domain/governance.ts';
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
  const voters = (h: LinkHalf | null): string =>
    h ? `${h.yes + h.no} of ${h.population} voted · ${percent(h)}% yes` : 'Not received yet';
</script>

<article id={domId} class={`link-vote ${cardClass}`}>
  <strong class="link-vote-title">{title}</strong>
  <span class="link-vote-meta">Proposed by {proposer}</span>
  {#if request.summary}<p class="link-vote-summary">{request.summary}</p>{/if}
  {#each [request.thisSide, request.otherSide] as side (side.projectId)}
    {@const half = side.half}
    {@const voted = half?.myVote ?? null}
    <div class="link-vote-side">
      <div class="link-vote-line">
        <b>{side === request.thisSide ? 'This project' : side.projectTitle}</b>
        <span>{voters(half)}</span>
      </div>
      <div class="link-vote-bar" class:done={half?.status === 'Approved'}>
        <i style={`width: ${Math.max(percent(half), 3)}%`}></i>
      </div>
      <span class="link-vote-note">
        {half?.status === 'Open' ? voteRequirement(half) : resultNote(half)}
      </span>
      {#if onVote && (side.canVote || voted)}
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
    </div>
  {/each}
</article>
