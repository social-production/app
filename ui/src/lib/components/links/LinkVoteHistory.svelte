<script lang="ts">
  import {
    sideVoteLines,
    statusLabel,
    targetMetaRows,
    type LinkRequest,
  } from '#lib/domain/links.ts';
  import { modeAccent } from '#lib/domain/projects.ts';

  import LinkCard from './LinkCard.svelte';
  import LinkVotePanel from './LinkVotePanel.svelte';

  import '#lib/components/styles/links.css';

  /** web's DetailLinkVoteHistorySection: resolved create and sever votes, folded away. */
  interface Props {
    requests: LinkRequest[];
    names: Record<string, string>;
  }

  let { requests, names }: Props = $props();

  let archiveOpen = $state(false);
  let expanded = $state<Record<string, boolean>>({});

  function tone(r: LinkRequest): 'approved' | 'blocked' | 'pending' {
    if (r.status === 'Approved') return 'approved';
    return r.status === 'Rejected' ? 'blocked' : 'pending';
  }
</script>

{#if requests.length > 0}
  <section class="vote-archive">
    <div class="archive-heading">
      <div class="heading-copy">
        <h2>Past link votes</h2>
        <p>Resolved create and sever votes stay here for context.</p>
      </div>
      <button class="archive-toggle" type="button" onclick={() => (archiveOpen = !archiveOpen)}>
        {archiveOpen ? 'Hide past link votes' : `Show past link votes (${requests.length})`}
      </button>
    </div>

    {#if archiveOpen}
      <div class="request-list">
        {#each requests as request (request.id)}
          {@const metaRows = targetMetaRows(request.target)}
          <LinkCard
            title={request.target.title}
            metaLine={`${request.sever ? 'Sever vote' : 'Link vote'} · ${names[request.proposedBy] ?? '…'}`}
            sideVoteLines={sideVoteLines(request)}
            outcomeLabel={statusLabel(request)}
            outcomeTone={tone(request)}
            accent={modeAccent(request.target.mode)}
            projectId={request.target.id}
            expanded={!!expanded[request.id]}
            domId={`link-request-${request.id}`}
            onToggle={() => (expanded = { ...expanded, [request.id]: !expanded[request.id] })}
          >
            <p>{request.summary}</p>
            {#if metaRows.length > 0}
              <div class="detail-meta">
                {#each metaRows as row (row)}<span>{row}</span>{/each}
              </div>
            {/if}
            <div class="vote-grid">
              {#each [request.thisSide, request.otherSide] as side (side.projectId)}
                <LinkVotePanel {side} showRequirement={false} />
              {/each}
            </div>
          </LinkCard>
        {/each}
      </div>
    {/if}
  </section>
{/if}
