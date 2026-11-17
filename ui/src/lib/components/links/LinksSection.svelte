<script lang="ts">
  import type { Snippet } from 'svelte';

  import { shortDate } from '#lib/domain/botReview.ts';
  import type { VoteChoice } from '#lib/domain/governance.ts';
  import type { LinkSide, ProjectLink } from '#lib/domain/links.ts';

  import { focusLinkCard } from './focusLinkCard.ts';
  import LinkCard from './LinkCard.svelte';
  import LinkVoteCard from './LinkVoteCard.svelte';

  import '#lib/components/styles/links.css';

  /** web's DetailLinksSection for active links: one row each (Polish Pass mockups); a row opens
   * to its details and "End this link", or to the vote that would end it. */
  interface Props {
    items: ProjectLink[];
    names: Record<string, string>;
    allowSever: boolean;
    /** The link or request id an action is running for. */
    pendingId: string | null;
    onSever: (link: ProjectLink) => void;
    onVote: (requestId: string, side: LinkSide, choice: VoteChoice) => void;
    /** A sever request a link asks to show (web's `?linkRequest=…`). */
    highlight?: string | null;
    /** "Propose a link", at the right of the heading (Polish Pass mockups §6). */
    action?: Snippet | undefined;
  }

  let {
    items,
    names,
    allowSever,
    pendingId,
    onSever,
    onVote,
    highlight = null,
    action,
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
</script>

<section class="network-stack">
  <div class="links-head">
    <h2 class="links-heading">
      Linked{#if items.length > 0}<span class="links-count">{items.length}</span>{/if}
    </h2>
    {@render action?.()}
  </div>
  {#if items.length === 0}
    <p class="links-quiet">Not linked to anything yet.</p>
  {:else}
    <div class="link-list">
      {#each items as item (item.id)}
        {@const target = item.create.target}
        <LinkCard
          title={target.title}
          metaLine={`${target.mode === 'event' ? 'Event' : 'Project'} · linked ${shortDate(item.create.createdAt / 1000)}`}
          icon={target.mode === 'event' ? 'event' : 'cube'}
          voteLine={item.sever ? 'A vote to end this link is open' : null}
          projectId={target.id}
          expanded={!!expanded[item.id]}
          domId={item.sever ? `link-request-${item.sever.id}` : `link-${item.id}`}
          onToggle={() => (expanded = { ...expanded, [item.id]: !expanded[item.id] })}
        >
          <p class="detail-description">{target.description || item.create.summary}</p>
          {#if item.sever}
            {@const sever = item.sever}
            <LinkVoteCard
              request={sever}
              proposer={names[sever.proposedBy] ?? '…'}
              yesLabel="Remove link"
              noLabel="Keep link"
              danger
              pending={pendingId === sever.id}
              onVote={(side, choice) => onVote(sever.id, side, choice)}
            />
          {:else if allowSever}
            <button
              class="danger-button end-link"
              disabled={pendingId === item.id}
              type="button"
              onclick={() => onSever(item)}>End this link</button
            >
          {/if}
        </LinkCard>
      {/each}
    </div>
  {/if}
</section>
