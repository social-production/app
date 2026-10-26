<script lang="ts">
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
    children: Snippet;
  }

  let {
    href = null,
    tone = 'public',
    accent,
    compact = false,
    clampExcerpts = true,
    children,
  }: Props = $props();
</script>

<article
  class="surface clickable tone-{tone}"
  class:compact
  class:has-accent={!!accent}
  style={accent ? `--row-accent: ${accent};` : undefined}
>
  {#if href}
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href arrives resolved (ResolvedPathname) -->
    <a aria-label="Open item" class="surface-link" {href}></a>
  {/if}
  <div class="content" class:clamp-excerpts={clampExcerpts} class:content-interactive={!href}>
    {@render children()}
  </div>
</article>
