<script lang="ts">
  import { modeAccent, takesSignals, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import ContentMetaRow from './ContentMetaRow.svelte';
  import CountPill from './CountPill.svelte';
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

  const href = $derived(resolve('/projects/[id]', { id: card.id }));
</script>

<div class="personal-activity-card">
  <FeedSurface {href} tone="personal" accent={modeAccent(card.mode)}>
    <div class="card-header">
      {#if card.tags.length > 0}
        <FeedCardTop stage={card.phase} tags={card.tags} />
      {/if}
      <!-- Without tags the stage joins the author's row (web's header-row + header-end, as its
           private items use), rather than sitting alone above it. -->
      <div class="header-row">
        <PersonalFeedIdentity
          name={authorName}
          href={resolve('/profile/[agent]', { agent: card.creator })}
        />
        {#if card.tags.length === 0 && card.phase}
          <div class="header-end"><span class="feed-card-stage">{card.phase}</span></div>
        {/if}
      </div>
    </div>
    <a class="title" href={resolve('/projects/[id]', { id: card.id })}>{card.title}</a>
    <p class="body">{card.summary}</p>
    {#if card.latestUpdate}
      <p class="latest-summary">Latest: {card.latestUpdate}</p>
    {/if}
    <div class="footer">
      <div class="engagement-row feed-corner-actions">
        <SignalStrip
          corner
          support={card.demand}
          oppose={card.opposition}
          mine={null}
          disabled={!takesSignals(card.mode)}
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
          timeOnly
          {authorName}
          authorHref={resolve('/profile/[agent]', { agent: card.creator })}
          createdAt={card.createdAt}
          updatedAt={card.lastActivityAt}
        />
      </div>
    </div>
  </FeedSurface>
</div>
