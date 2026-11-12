<script lang="ts">
  import CardReport from '#lib/components/moderation/CardReport.svelte';
  import { localDateTime, signupSummary } from '#lib/domain/helpRequests.ts';
  import { modeAccent, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import FeedActions from './FeedActions.svelte';
  import FeedCardTop from './FeedCardTop.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import PersonalFeedIdentity from './PersonalFeedIdentity.svelte';
  import VoteStrip from './VoteStrip.svelte';

  import '#lib/components/styles/cards.css';

  /**
   * web's PersonalHelpRequestCard (the personal feed and profiles): tags, the asker's identity
   * row, when and where, who signed up, votes and comments, and the clock alone in the footer.
   * web's ⋮ report menu comes with moderation.
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
</script>

<div class="personal-help-request-card">
  <FeedSurface
    {href}
    tone="personal"
    accent={modeAccent(card.mode)}
    contentRestricted={card.moderation === 'hidden'}
  >
    <div class="card-header">
      <FeedCardTop
        tags={card.tags}
        stage=""
        mode={card.mode}
        createdAt={card.createdAt}
        updatedAt={card.lastActivityAt}><CardReport {card} /></FeedCardTop
      >
      <div class="header-row">
        <PersonalFeedIdentity
          name={authorName}
          href={resolve('/profile/[agent]', { agent: card.creator })}
        />
      </div>
    </div>
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
