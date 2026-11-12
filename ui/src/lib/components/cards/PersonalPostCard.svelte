<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import CardReport from '#lib/components/moderation/CardReport.svelte';
  import { modeAccent, type ProjectCard, type Stance } from '#lib/domain/projects.ts';
  import { describeUpdate, relativeCompact } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';

  import FeedActions from './FeedActions.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import LinkedPostBody from './LinkedPostBody.svelte';
  import VoteStrip from './VoteStrip.svelte';

  import '#lib/components/styles/cards.css';

  /**
   * web's PersonalPostCard (personal feed and profiles, ADR 0070): laid out as a
   * Twitter row (ADR 0096): the avatar in its own column, then the name and clock, the body
   * (`LinkedPostBody`: its links live), web's ⋮, and the action row.
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
    <div class="post-row">
      <AvatarBadge name={authorName} />
      <div class="post-main">
        <div class="post-head">
          <a class="header-name" href={resolve('/profile/[agent]', { agent: card.creator })} title={authorName}>{authorName}</a>
          <span class="feed-card-time" title={describeUpdate(card.createdAt, null)}
            >{relativeCompact(card.createdAt)}</span
          >
          <CardReport {card} />
        </div>
        <LinkedPostBody body={card.summary} />
        <FeedActions
          commentHref={`${href}#comments`}
          commentCount={card.commentCount}
          shareHref={href}
          shareTitle={card.summary}
        >
          <VoteStrip bare support={card.demand} oppose={card.opposition} mine={null} {onSignal} />
        </FeedActions>
      </div>
    </div>
  </FeedSurface>
</div>
