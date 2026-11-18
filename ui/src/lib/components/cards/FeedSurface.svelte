<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { FeedContext } from '#lib/domain/homeFeed.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import type { Snippet } from 'svelte';

  import type { ResolvedPathname } from '$app/types';

  import '#lib/components/styles/cards.css';

  /** web's FeedSurface: one feed row; the whole row links to `href`. */
  interface Props {
    /** Left out when the slot handles the click (web's `interactive` rows, e.g. notifications). */
    href?: ResolvedPathname | null;
    /** CSS colour for the left accent bar (web's surface type accent). */
    accent: string | null;
    /** web's tones: public-feed rows, or personal-feed rows (author first). */
    tone?: 'public' | 'personal';
    /** Tighter padding for dense rows such as notifications. */
    compact?: boolean;
    /** Feed cards clamp body/summary; notifications show it whole. */
    clampExcerpts?: boolean;
    /** web's serious-harm restriction (ADR 0071): blurred behind a warning until revealed. */
    contentRestricted?: boolean;
    /** Who made it, on Home for someone you follow: "Maya Kim proposed this" (ADR 0107). */
    context?: FeedContext | null;
    children: Snippet;
  }

  let {
    href = null,
    tone = 'public',
    accent,
    compact = false,
    clampExcerpts = true,
    contentRestricted = false,
    context = null,
    children,
  }: Props = $props();

  let revealed = $state(false);
</script>

<article
  class="surface clickable tone-{tone}"
  class:compact
  class:has-accent={!!accent}
  class:content-restricted={contentRestricted}
  style={accent ? `--row-accent: ${accent};` : undefined}
>
  {#if href}
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href arrives resolved (ResolvedPathname) -->
    <a aria-label={t('Open item')} class="surface-link" {href}></a>
  {/if}
  {#if contentRestricted}
    <div class="restriction-banner">
      <span>{t('Serious harm report — content blurred until you choose to reveal it.')}</span>
      <button
        class="reveal-btn"
        type="button"
        onclick={(event) => {
          event.stopPropagation();
          revealed = !revealed;
        }}
      >
        {revealed ? t('Hide again') : t('Reveal')}
      </button>
    </div>
  {/if}
  <div
    class="content"
    class:blurred={contentRestricted && !revealed}
    class:clamp-excerpts={clampExcerpts}
    class:content-interactive={!href}
  >
    {#if context}
      <p class="feed-context">
        <AvatarBadge size="xs" name={context.name} avatar={context.avatar} /><span
          >{context.text}</span
        >
      </p>
    {/if}
    {@render children()}
  </div>
</article>
