<script lang="ts">
  import CardReport from '#lib/components/moderation/CardReport.svelte';
  import { modeAccent, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import ContentMetaRow from './ContentMetaRow.svelte';
  import CountPill from './CountPill.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import LinkedPostBody from './LinkedPostBody.svelte';
  import PersonalFeedIdentity from './PersonalFeedIdentity.svelte';
  import VoteStrip from './VoteStrip.svelte';

  import '#lib/components/styles/cards.css';

  /**
   * web's PersonalPostCard (personal feed and profiles, ADR 0070): the author's identity row,
   * the post's body (`LinkedPostBody`: its links live), web's ⋮, votes and comments, and the
   * clock.
   */
  interface Props {
    card: ProjectCard;
    authorName: string;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, authorName, onSignal }: Props = $props();

  const href = $derived(resolve('/posts/[id]', { id: card.id }));
</script>

<div class="personal-post-card">
  <FeedSurface
    {href}
    tone="personal"
    accent={modeAccent(card.mode)}
    contentRestricted={card.moderation === 'hidden'}
  >
    <div class="card-header">
      <div class="header-row">
        <PersonalFeedIdentity
          name={authorName}
          href={resolve('/profile/[agent]', { agent: card.creator })}
        />
        <CardReport {card} />
      </div>
    </div>
    <LinkedPostBody body={card.summary} />
    <div class="footer">
      <div class="engagement-row feed-corner-actions">
        <VoteStrip corner support={card.demand} oppose={card.opposition} mine={null} {onSignal} />
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved path + web's #comments -->
        <a class="comment-link" href={`${href}#comments`}>
          <CountPill
            label={`${card.commentCount} ${card.commentCount === 1 ? 'comment' : 'comments'}`}
          />
        </a>
      </div>
      <div class="footer-meta">
        <ContentMetaRow
          timeOnly
          authorName=""
          authorHref={resolve('/profile/[agent]', { agent: card.creator })}
          createdAt={card.createdAt}
        />
      </div>
    </div>
  </FeedSurface>
</div>
