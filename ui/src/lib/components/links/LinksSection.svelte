<script lang="ts">
  import type { VoteChoice } from '#lib/domain/governance.ts';
  import {
    sideVoteLines,
    tallyLabel,
    targetMetaRows,
    type LinkSide,
    type ProjectLink,
  } from '#lib/domain/links.ts';
  import { modeAccent } from '#lib/domain/projects.ts';

  import { focusLinkCard } from './focusLinkCard.ts';
  import LinkCard from './LinkCard.svelte';
  import LinkVotePanel from './LinkVotePanel.svelte';

  import '#lib/components/styles/links.css';

  /** web's DetailLinksSection: approved (or past) links; active ones can be put to a sever vote. */
  interface Props {
    title: string;
    description: string;
    items: ProjectLink[];
    emptyMessage: string;
    allowSever: boolean;
    /** The link or request id an action is running for. */
    pendingId: string | null;
    onSever?: (link: ProjectLink) => void;
    onVote?: (requestId: string, side: LinkSide, choice: VoteChoice) => void;
    /** A sever request a link asks to show (web's `?linkRequest=…`). */
    highlight?: string | null;
  }

  let {
    title,
    description,
    items,
    emptyMessage,
    allowSever,
    pendingId,
    onSever,
    onVote,
    highlight = null,
  }: Props = $props();

  let expanded = $state<Record<string, boolean>>({});
  let shown: string | null = null;

  $effect(() => {
    const match = highlight ? items.find((i) => i.sever?.id === highlight) : undefined;
    if (!match || !highlight || shown === highlight) return;
    shown = highlight;
    expanded = { ...expanded, [match.id]: true };
    void focusLinkCard(highlight);
  });

  function metaLine(item: ProjectLink): string {
    const label = item.sever ? tallyLabel(item.sever) : allowSever ? 'Linked' : 'Inactive';
    return `Project · ${label}`;
  }
</script>

<section class="network-stack">
  <div class="section-heading">
    <h2>{title}</h2>
    {#if description}<p>{description}</p>{/if}
  </div>

  {#if items.length === 0}
    <div class="empty-row">{emptyMessage}</div>
  {:else}
    <div class="link-list">
      {#each items as item (item.id)}
        {@const target = item.create.target}
        {@const metaRows = targetMetaRows(target)}
        <LinkCard
          title={target.title}
          metaLine={metaLine(item)}
          sideVoteLines={item.sever ? sideVoteLines(item.sever) : []}
          accent={modeAccent(target.mode)}
          projectId={target.id}
          expanded={!!expanded[item.id]}
          domId={item.sever ? `link-request-${item.sever.id}` : `link-${item.id}`}
          onToggle={() => (expanded = { ...expanded, [item.id]: !expanded[item.id] })}
        >
          <div class="target-detail">
            <p class="detail-description">{target.description || item.create.summary}</p>
            {#if metaRows.length > 0}
              <div class="detail-meta">
                {#each metaRows as row (row)}<span>{row}</span>{/each}
              </div>
            {/if}
          </div>
          {#if item.sever}
            {@const sever = item.sever}
            <div class="vote-block">
              <h3>Sever vote</h3>
              <p class="vote-summary">{sever.summary}</p>
              <div class="vote-grid">
                {#each [sever.thisSide, sever.otherSide] as side (side.projectId)}
                  <LinkVotePanel
                    {side}
                    yesLabel="Remove link"
                    noLabel="Keep link"
                    danger
                    pending={pendingId === sever.id}
                    onVote={onVote ? (s, choice) => onVote(sever.id, s, choice) : undefined}
                  />
                {/each}
              </div>
            </div>
          {:else if allowSever && onSever}
            <div class="sever-action">
              <button
                class="danger-button"
                disabled={pendingId === item.id}
                type="button"
                onclick={() => onSever(item)}>Start sever vote</button
              >
            </div>
          {/if}
        </LinkCard>
      {/each}
    </div>
  {/if}
</section>
