<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import CardFacts from '#lib/components/cards/CardFacts.svelte';
  import SurfaceTypeLabel from '#lib/components/rail/SurfaceTypeLabel.svelte';
  import type { CharterMode } from '#lib/domain/projects.ts';

  /**
   * The last step of a create flow (Polish Pass mockups, 2026-11-16): the card as it will
   * appear in the feed, drawn with the feed row's own glyph, type line and when / where.
   */
  interface Props {
    mode: CharterMode;
    /** The main channel or community, without its # or @. */
    scope: string;
    title: string;
    body: string;
    when?: string;
    place?: string;
    /** The picture chosen in the first step (ADR 0117): the row's tile, as in the feed. */
    picture?: string | null;
  }

  let { mode, scope, title, body, when = '', place = '', picture = null }: Props = $props();
</script>

<figure class="create-preview">
  <figcaption>{t('This is how it will look in the feed.')}</figcaption>
  <!-- `shell` so the feed row's own styles reach it inside the portaled sheet. -->
  <div class="shell create-preview-card">
    <div class="feed-card-top feed-lead">
      <span class="feed-lead-glyph"
        >{#if picture}<img class="feed-lead-picture" src={picture} alt="" />{:else}<SurfaceTypeLabel
            projectMode={mode}
            feedGlyph
          />{/if}</span
      >
      <div class="feed-lead-copy">
        {#if scope}<div class="feed-lead-tags">
            <span class="create-preview-scope">{scope}</span>
          </div>{/if}
        <div class="feed-lead-meta" class:alone={!scope}>
          <SurfaceTypeLabel projectMode={mode} />
          <span class="feed-card-time">{t('now')}</span>
        </div>
      </div>
    </div>
    <strong class="create-preview-title">{title || 'Untitled'}</strong>
    {#if body}<p class="create-preview-body">{body}</p>{/if}
    <CardFacts {when} {place} />
  </div>
</figure>
