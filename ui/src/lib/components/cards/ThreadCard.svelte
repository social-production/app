<script lang="ts">
  import CardReport from '#lib/components/moderation/CardReport.svelte';
  import { modeAccent, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import FeedActions from './FeedActions.svelte';
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
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, onSignal }: Props = $props();

  const href = $derived(resolve('/threads/[id]', { id: card.id }));
</script>

<div class="thread-card">
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
    <a class="title" href={resolve('/threads/[id]', { id: card.id })}>{card.title}</a>
    <p class="body">{card.summary}</p>
    <FeedActions
      commentHref={href}
      commentCount={card.commentCount}
      shareHref={href}
      shareTitle={card.title}
    >
      <VoteStrip bare support={card.demand} oppose={card.opposition} mine={null} {onSignal} />
    </FeedActions>
  </FeedSurface>
</div>
