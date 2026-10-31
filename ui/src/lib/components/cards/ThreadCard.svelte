<script lang="ts">
  import { modeAccent, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import ContentMetaRow from './ContentMetaRow.svelte';
  import CountPill from './CountPill.svelte';
  import FeedCardTop from './FeedCardTop.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import VoteStrip from './VoteStrip.svelte';

  import '#lib/components/styles/cards.css';

  /**
   * web's public-feed ThreadCard (ADR 0070): its channel and community tags, the title, the
   * body, votes and comments. web's ⋮ report menu comes with moderation.
   */
  interface Props {
    card: ProjectCard;
    authorName: string;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, authorName, onSignal }: Props = $props();

  const href = $derived(resolve('/threads/[id]', { id: card.id }));
</script>

<div class="thread-card">
  <FeedSurface {href} accent={modeAccent(card.mode)}>
    <FeedCardTop tags={card.tags} stage="" />
    <a class="title" href={resolve('/threads/[id]', { id: card.id })}>{card.title}</a>
    <p class="body">{card.summary}</p>
    <div class="footer">
      <div class="engagement-row feed-corner-actions">
        <VoteStrip corner support={card.demand} oppose={card.opposition} mine={null} {onSignal} />
        <a class="comment-link" href={resolve('/threads/[id]', { id: card.id })}>
          <CountPill
            label={`${card.commentCount} ${card.commentCount === 1 ? 'comment' : 'comments'}`}
          />
        </a>
      </div>
      <div class="footer-meta">
        <ContentMetaRow
          {authorName}
          authorHref={resolve('/profile/[agent]', { agent: card.creator })}
          createdAt={card.createdAt}
        />
      </div>
    </div>
  </FeedSurface>
</div>
