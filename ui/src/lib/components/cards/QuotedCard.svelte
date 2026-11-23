<script lang="ts">
  import SurfaceTypeLabel from '#lib/components/rail/SurfaceTypeLabel.svelte';
  import { spaceKind } from '#lib/domain/commentNotes.ts';
  import { tagRef } from '#lib/domain/scopes.ts';

  import { quoteSource, type QuotedSpace } from './quoteSource.ts';

  /**
   * The quoted card under a quote post's words (Owner Notes Mockups §6), drawn like the card
   * under a comment on a profile (`CommentActivityCard`).
   */
  let { id, href }: { id: string; href: string } = $props();

  const source = quoteSource();
  let space = $state<QuotedSpace | null>(null);

  $effect(() => {
    void source?.summary(id).then(
      (found) => (space = found),
      () => (space = null),
    );
  });

  const scope = $derived(space?.tag ? tagRef(space.tag).label : '');
</script>

<span class="comment-activity-card quoted-card">
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- a path written in the post -->
  <a class="comment-quote" {href} onclick={(e) => e.stopPropagation()}>
    <span class="comment-quote-glyph"
      ><SurfaceTypeLabel projectMode={space?.mode ?? 'productive'} feedGlyph /></span
    >
    <span class="comment-quote-copy">
      <small>{scope ? `${scope} · ` : ''}{spaceKind(space?.mode ?? 'productive')}</small>
      <strong>{space?.title ?? '…'}</strong>
    </span>
  </a>
</span>
