<script lang="ts">
  import type { VoteChoice } from '#lib/domain/governance.ts';
  import {
    sideVoteLines,
    tallyLabel,
    targetMetaRows,
    type LinkRequest,
    type LinkSide,
  } from '#lib/domain/links.ts';
  import { modeAccent } from '#lib/domain/projects.ts';

  import { focusLinkCard } from './focusLinkCard.ts';
  import LinkCard from './LinkCard.svelte';
  import LinkVotePanel from './LinkVotePanel.svelte';

  import '#lib/components/styles/links.css';

  /** web's DetailLinkRequestsSection: open link votes; both projects must approve. */
  interface Props {
    requests: LinkRequest[];
    names: Record<string, string>;
    canPropose: boolean;
    pendingId: string | null;
    onCompose: () => void;
    onVote: (requestId: string, side: LinkSide, choice: VoteChoice) => void;
    /** A request a link asks to show (web's `?linkRequest=…`): expanded and scrolled to. */
    highlight?: string | null;
  }

  let {
    requests,
    names,
    canPropose,
    pendingId,
    onCompose,
    onVote,
    highlight = null,
  }: Props = $props();

  let expanded = $state<Record<string, boolean>>({});
  let shown: string | null = null;

  $effect(() => {
    if (!highlight || shown === highlight || !requests.some((r) => r.id === highlight)) return;
    shown = highlight;
    expanded = { ...expanded, [highlight]: true };
    void focusLinkCard(highlight);
  });
</script>

<section class="request-stack">
  <div class="section-heading">
    <div class="heading-copy">
      <h2>Pending link requests</h2>
      <p>
        Search for a project or event, explain why they should be linked, then start a vote. Both
        sides must approve.
      </p>
    </div>
    {#if canPropose}
      <button class="propose-link-button" type="button" onclick={onCompose}>
        Propose a new link
      </button>
    {/if}
  </div>

  {#if requests.length === 0}
    <div class="empty-row">No pending link votes.</div>
  {:else}
    <div class="request-list">
      {#each requests as request (request.id)}
        {@const target = request.target}
        {@const metaRows = targetMetaRows(target)}
        <LinkCard
          title={target.title}
          metaLine={`Project · ${tallyLabel(request)} · ${names[request.proposedBy] ?? '…'}`}
          sideVoteLines={sideVoteLines(request)}
          accent={modeAccent(target.mode)}
          projectId={target.id}
          expanded={!!expanded[request.id]}
          domId={`link-request-${request.id}`}
          onToggle={() => (expanded = { ...expanded, [request.id]: !expanded[request.id] })}
        >
          <div class="target-detail">
            <p>{target.description || request.summary}</p>
            {#if metaRows.length > 0}
              <div class="detail-meta">
                {#each metaRows as row (row)}<span>{row}</span>{/each}
              </div>
            {/if}
          </div>
          <div class="vote-grid">
            {#each [request.thisSide, request.otherSide] as side (side.projectId)}
              <LinkVotePanel
                {side}
                pending={pendingId === request.id}
                onVote={(s, choice) => onVote(request.id, s, choice)}
              />
            {/each}
          </div>
        </LinkCard>
      {/each}
    </div>
  {/if}
</section>
