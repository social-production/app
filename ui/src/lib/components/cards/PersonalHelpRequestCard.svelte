<script lang="ts">
  import { localDateTime, signupSummary } from '#lib/domain/helpRequests.ts';
  import { modeAccent, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import ContentMetaRow from './ContentMetaRow.svelte';
  import CountPill from './CountPill.svelte';
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
  <FeedSurface {href} tone="personal" accent={modeAccent(card.mode)}>
    <div class="card-header">
      <FeedCardTop tags={card.tags} stage="" />
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
