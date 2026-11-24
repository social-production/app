<script lang="ts">
  import type { FeedContext } from '#lib/domain/homeFeed.ts';
  import CardReport from '#lib/components/moderation/CardReport.svelte';
  import { modeAccent, takesSignals, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import CardFacts from './CardFacts.svelte';
  import FeedActions from './FeedActions.svelte';
  import FeedCardTop from './FeedCardTop.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import NextSession from './NextSession.svelte';
  import SignalStrip from './SignalStrip.svelte';

  /** web's public-feed ProjectCard (classes styled by styles/cards.css, `.project-card`). */
  interface Props {
    card: ProjectCard;
    onSignal: (next: Stance | null) => Promise<void>;
    /** Home's \"Maya Kim proposed this\" line. */
    context?: FeedContext | null;
  }

  let { card, onSignal, context = null }: Props = $props();

  const href = $derived(resolve('/projects/[id]', { id: card.id }));

  import '#lib/components/styles/cards.css';
</script>

<div class="project-card">
  <FeedSurface
    {context}
    {href}
    accent={modeAccent(card.mode)}
    contentRestricted={card.moderation === 'hidden'}
  >
    <FeedCardTop
      picture={card.picture ?? null}
      tags={card.tags}
      stage={card.phase}
      mode={card.mode}
      createdAt={card.createdAt}
      updatedAt={card.lastActivityAt}
      distanceKm={card.distanceKm ?? null}><CardReport {card} {href} /></FeedCardTop
    >
    <a class="title" href={resolve('/projects/[id]', { id: card.id })}>{card.title}</a>
    <p class="summary">{card.summary}</p>
    {#if card.latestUpdate}
      <p class="latest-summary">Latest: {card.latestUpdate}</p>
    {/if}
    <CardFacts place={card.locationLabel} target={href} />
    {#if card.nextActivity}<NextSession next={card.nextActivity} {href} />{/if}
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
        disabled={!takesSignals(card.mode)}
        {onSignal}
      />
    </FeedActions>
  </FeedSurface>
</div>
