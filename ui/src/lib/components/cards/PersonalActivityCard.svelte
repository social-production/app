<script lang="ts">
  import CardReport from '#lib/components/moderation/CardReport.svelte';
  import { modeAccent, takesSignals, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import { spaceHref } from './spaceHref.ts';
  import FeedActions from './FeedActions.svelte';
  import FeedCardTop from './FeedCardTop.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import PersonalFeedIdentity from './PersonalFeedIdentity.svelte';
  import SignalStrip from './SignalStrip.svelte';

  import '#lib/components/styles/cards.css';

  /**
   * web's PersonalActivityCard for a project (the personal feed and profiles): stage and tags,
   * then the author's identity row, the title, and the clock alone in the footer.
   */
  interface Props {
    card: ProjectCard;
    authorName: string;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, authorName, onSignal }: Props = $props();

  const href = $derived(spaceHref(card.mode, card.id));
</script>

<div class="personal-activity-card">
  <FeedSurface
    {href}
    tone="personal"
    accent={modeAccent(card.mode)}
    contentRestricted={card.moderation === 'hidden'}
  >
    <div class="card-header">
      <FeedCardTop
        stage={card.phase}
        tags={card.tags}
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
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- spaceHref resolves it -->
    <a class="title" {href}>{card.title}</a>
    <p class="body">{card.summary}</p>
    {#if card.latestUpdate}
      <p class="latest-summary">Latest: {card.latestUpdate}</p>
    {/if}
    <FeedActions
      commentHref={`${href}?tab=chat`}
      commentCount={card.commentCount}
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
