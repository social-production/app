<script lang="ts">
  import CardReport from '#lib/components/moderation/CardReport.svelte';
  import { localDateTime, signupSummary } from '#lib/domain/helpRequests.ts';
  import { modeAccent, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import FeedActions from './FeedActions.svelte';
  import FeedCardTop from './FeedCardTop.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import VoteStrip from './VoteStrip.svelte';

  /**
   * web's public-feed HelpRequestCard (styles/cards.css, `.help-request-card`): when and where,
   * who signed up, votes and comments (ADR 0067). web's ⋮ report menu comes with moderation.
   */
  interface Props {
    card: ProjectCard;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, onSignal }: Props = $props();

  const href = $derived(resolve('/help-requests/[id]', { id: card.id }));
  const when = $derived(localDateTime(card.help?.neededAt ?? null));
  const signups = $derived(
    signupSummary(card.help?.signupCount ?? null, card.help?.slotsNeeded ?? null),
  );

  import '#lib/components/styles/cards.css';
</script>

<div class="help-request-card">
  <FeedSurface
    {href}
    accent={modeAccent(card.mode)}
    contentRestricted={card.moderation === 'hidden'}
  >
    <FeedCardTop
      tags={card.tags}
      stage=""
      mode={card.mode}
      createdAt={card.createdAt}
      updatedAt={card.lastActivityAt}><CardReport {card} {href} /></FeedCardTop
    >
    <a class="title" href={resolve('/help-requests/[id]', { id: card.id })}>{card.title}</a>
    <p class="body">{card.summary}</p>
    {#if when || card.locationLabel}
      <p class="location">{[when, card.locationLabel].filter(Boolean).join(' · ')}</p>
    {/if}
    {#if signups}<p class="signup-summary">{signups}</p>{/if}
    <FeedActions
      commentHref={`${href}?tab=chat`}
      commentCount={card.commentCount}
      shareHref={href}
      shareTitle={card.title}
    >
      <VoteStrip bare support={card.demand} oppose={card.opposition} mine={null} {onSignal} />
    </FeedActions>
  </FeedSurface>
</div>
