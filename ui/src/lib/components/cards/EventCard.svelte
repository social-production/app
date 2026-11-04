<script lang="ts">
  import CardReport from '#lib/components/moderation/CardReport.svelte';
  import { implementedLabel } from '#lib/domain/eventSchedule.ts';
  import { localDateTime } from '#lib/domain/helpRequests.ts';
  import { modeAccent, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import ContentMetaRow from './ContentMetaRow.svelte';
  import CountPill from './CountPill.svelte';
  import FeedCardTop from './FeedCardTop.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import SignalStrip from './SignalStrip.svelte';

  /**
   * web's public-feed EventCard (styles/cards.css, `.event-card`). An event is a listing with
   * mode "event" (ADR 0057); the place line starts with the leading plan's start time.
   */
  interface Props {
    card: ProjectCard;
    authorName: string;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, authorName, onSignal }: Props = $props();

  const href = $derived(resolve('/events/[id]', { id: card.id }));
  const closed = $derived(card.phase === 'Closed');
  const when = $derived(localDateTime(card.scheduledAt ?? null));
  const place = $derived([when, implementedLabel(card.locationLabel)].filter(Boolean).join(' · '));

  import '#lib/components/styles/cards.css';
</script>

<div class="event-card">
  <FeedSurface
    {href}
    accent={modeAccent(card.mode)}
    contentRestricted={card.moderation === 'hidden'}
  >
    <FeedCardTop tags={card.tags} stage={card.phase}><CardReport {card} {href} /></FeedCardTop>
    <a class="title" href={resolve('/events/[id]', { id: card.id })}>{card.title}</a>
    <p class="body">{card.summary}</p>
    {#if card.latestUpdate}
      <p class="latest-summary">Latest: {card.latestUpdate}</p>
    {/if}
    {#if place}
      <p class="location">{place}</p>
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
