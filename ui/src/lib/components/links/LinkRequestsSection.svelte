<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { VoteChoice } from '#lib/domain/governance.ts';
  import type { LinkRequest, LinkSide } from '#lib/domain/links.ts';

  import { focusLinkCard } from './focusLinkCard.ts';
  import LinkVoteCard from './LinkVoteCard.svelte';

  import '#lib/components/styles/links.css';

  /** web's DetailLinkRequestsSection: open link votes, each one card; both projects must
   * approve. "Propose a link" sits at the top of the tab (Polish Pass mockups). */
  interface Props {
    requests: LinkRequest[];
    names: Record<string, string>;
    pendingId: string | null;
    onVote: (requestId: string, side: LinkSide, choice: VoteChoice) => void;
    /** A request a link asks to show (web's `?linkRequest=…`): scrolled to. */
    highlight?: string | null;
  }

  let { requests, names, pendingId, onVote, highlight = null }: Props = $props();

  let shown: string | null = null;

  $effect(() => {
    if (!highlight || shown === highlight || !requests.some((r) => r.id === highlight)) return;
    shown = highlight;
    void focusLinkCard(highlight);
  });
</script>

<section class="request-stack">
  <h2 class="links-heading">
    {t('Votes open')}{#if requests.length > 0}<span class="links-count">{requests.length}</span
      >{/if}
  </h2>
  {#if requests.length === 0}
    <p class="links-quiet">{t('No votes open.')}</p>
  {:else}
    <div class="request-list">
      {#each requests as request (request.id)}
        <LinkVoteCard
          {request}
          proposer={names[request.proposedBy] ?? '…'}
          cardClass="link-card expanded"
          domId={`link-request-${request.id}`}
          pending={pendingId === request.id}
          onVote={(side, choice) => onVote(request.id, side, choice)}
        />
      {/each}
    </div>
  {/if}
</section>
