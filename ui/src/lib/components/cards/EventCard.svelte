<script lang="ts">
  import { modeAccent, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import ContentMetaRow from './ContentMetaRow.svelte';
  import CountPill from './CountPill.svelte';
  import FeedCardTop from './FeedCardTop.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import SignalStrip from './SignalStrip.svelte';

  /**
   * web's public-feed EventCard (styles/cards.css, `.event-card`). An event is a listing with
   * mode "event" (ADR 0057); its time joins the place line once the index carries the
   * leading plan's schedule.
   */
  interface Props {
    card: ProjectCard;
    authorName: string;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, authorName, onSignal }: Props = $props();

  const href = $derived(resolve('/events/[id]', { id: card.id }));
  const closed = $derived(card.phase === 'Closed');

  import '#lib/components/styles/cards.css';
</script>

<div class="event-card">
  <FeedSurface {href} accent={modeAccent(card.mode)}>
    <FeedCardTop tags={card.tags} stage={card.phase} />
    <a class="title" href={resolve('/events/[id]', { id: card.id })}>{card.title}</a>
    <p class="body">{card.summary}</p>
    {#if card.latestUpdate}
      <p class="latest-summary">Latest: {card.latestUpdate}</p>
    {/if}
    {#if card.locationLabel}
      <p class="location">{card.locationLabel}</p>
    {/if}
    <div class="footer">
      <div class="engagement-row feed-corner-actions">
        <SignalStrip
          corner
          support={card.demand}
          oppose={card.opposition}
          mine={null}
          disabled={closed}
          {onSignal}
        />
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
          memberCount={card.members}
          createdAt={card.createdAt}
          updatedAt={card.lastActivityAt}
        />
      </div>
    </div>
  </FeedSurface>
</div>
