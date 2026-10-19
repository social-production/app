<script lang="ts">
  import { modeAccent, takesSignals, type ProjectCard, type Stance } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  import ContentMetaRow from './ContentMetaRow.svelte';
  import FeedCardTop from './FeedCardTop.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import SignalStrip from './SignalStrip.svelte';

  /** web's public-feed ProjectCard (classes styled by styles/cards.css, `.project-card`). */
  interface Props {
    card: ProjectCard;
    authorName: string;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, authorName, onSignal }: Props = $props();

  const href = $derived(resolve('/projects/[id]', { id: card.id }));

  import '#lib/components/styles/cards.css';
</script>

<div class="project-card">
  <FeedSurface {href} accent={modeAccent(card.mode)}>
    <FeedCardTop tags={card.tags} stage={card.phase} />
    <a class="title" href={resolve('/projects/[id]', { id: card.id })}>{card.title}</a>
    <p class="summary">{card.summary}</p>
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
      </div>
      <div class="footer-meta">
        <ContentMetaRow
          {authorName}
          authorHref={resolve('/profile/[agent]', { agent: card.creator })}
          memberCount={card.members}
          createdAt={card.createdAt}
          updatedAt={card.lastActivityAt}
        />
      </div>
    </div>
  </FeedSurface>
</div>
