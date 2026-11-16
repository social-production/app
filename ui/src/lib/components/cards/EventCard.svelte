<script lang="ts">
  import CardReport from '#lib/components/moderation/CardReport.svelte';
  import { localDateTime } from '#lib/domain/helpRequests.ts';
  import { modeAccent, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import CardFacts from './CardFacts.svelte';
  import FeedActions from './FeedActions.svelte';
  import FeedCardTop from './FeedCardTop.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import SignalStrip from './SignalStrip.svelte';

  /**
   * web's public-feed EventCard (styles/cards.css, `.event-card`). An event is a listing with
   * mode "event" (ADR 0057); its leading plan's start time and place sit on their own rows.
   */
  interface Props {
    card: ProjectCard;
    onSignal: (next: Stance | null) => Promise<void>;
    /** Home's \"Maya Kim proposed this\" line. */
    context?: string;
  }

  let { card, onSignal, context = '' }: Props = $props();

  const href = $derived(resolve('/events/[id]', { id: card.id }));
  const closed = $derived(card.phase === 'Closed');
  const when = $derived(localDateTime(card.scheduledAt ?? null));

  import '#lib/components/styles/cards.css';
</script>

<div class="event-card">
  <FeedSurface
    {context}
    {href}
    accent={modeAccent(card.mode)}
    contentRestricted={card.moderation === 'hidden'}
  >
    <FeedCardTop
      tags={card.tags}
      stage={card.phase}
      mode={card.mode}
      createdAt={card.createdAt}
      updatedAt={card.lastActivityAt}><CardReport {card} {href} /></FeedCardTop
    >
    <a class="title" href={resolve('/events/[id]', { id: card.id })}>{card.title}</a>
    <p class="body">{card.summary}</p>
    {#if card.latestUpdate}
      <p class="latest-summary">Latest: {card.latestUpdate}</p>
    {/if}
    <CardFacts {when} place={card.locationLabel} />
    <FeedActions
      commentHref={`${href}?tab=chat`}
      commentCount={card.commentCount}
      memberCount={card.members}
      shareHref={href}
      shareTitle={card.title}
    >
      <SignalStrip
        bare
        support={card.demand}
        oppose={card.opposition}
        mine={null}
        disabled={closed}
        {onSignal}
      />
    </FeedActions>
  </FeedSurface>
</div>
