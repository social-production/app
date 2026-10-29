<script lang="ts">
  import { localDateTime, signupSummary } from '#lib/domain/helpRequests.ts';
  import { modeAccent, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import ContentMetaRow from './ContentMetaRow.svelte';
  import CountPill from './CountPill.svelte';
  import FeedCardTop from './FeedCardTop.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import VoteStrip from './VoteStrip.svelte';

  /**
   * web's public-feed HelpRequestCard (styles/cards.css, `.help-request-card`): when and where,
   * who signed up, votes and comments (ADR 0067). web's ⋮ report menu comes with moderation.
   */
  interface Props {
    card: ProjectCard;
    authorName: string;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, authorName, onSignal }: Props = $props();

  const href = $derived(resolve('/help-requests/[id]', { id: card.id }));
  const when = $derived(localDateTime(card.help?.neededAt ?? null));
  const signups = $derived(
    signupSummary(card.help?.signupCount ?? null, card.help?.slotsNeeded ?? null),
  );

  import '#lib/components/styles/cards.css';
</script>

<div class="help-request-card">
  <FeedSurface {href} accent={modeAccent(card.mode)}>
    <FeedCardTop tags={card.tags} stage="" />
    <a class="title" href={resolve('/help-requests/[id]', { id: card.id })}>{card.title}</a>
    <p class="body">{card.summary}</p>
    {#if when || card.locationLabel}
      <p class="location">{[when, card.locationLabel].filter(Boolean).join(' · ')}</p>
    {/if}
    {#if signups}<p class="signup-summary">{signups}</p>{/if}
    <div class="footer">
      <div class="engagement-row feed-corner-actions">
        <VoteStrip corner support={card.demand} oppose={card.opposition} mine={null} {onSignal} />
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved path + web's ?tab=chat -->
        <a class="comment-link" href={`${href}?tab=chat`}>
          <CountPill
            label={`${card.commentCount} ${card.commentCount === 1 ? 'comment' : 'comments'}`}
          />
        </a>
      </div>
      <div class="footer-meta">
        <ContentMetaRow
          {authorName}
          authorHref={resolve('/profile/[agent]', { agent: card.creator })}
          createdAt={card.createdAt}
          updatedAt={card.lastActivityAt}
        />
      </div>
    </div>
  </FeedSurface>
</div>
