<script lang="ts">
  import type { Snippet } from 'svelte';

  import type { ResolvedPathname } from '$app/types';

  import '#lib/components/styles/cards.css';

  /** web's FeedSurface: one feed row; with `href` the whole row links there. */
  interface Props {
    href?: ResolvedPathname;
    /** CSS colour for the left accent bar (web's surface type accent). */
    accent: string;
    /** Feed cards clamp excerpts; detail surfaces that embed discussion don't. */
    clamp?: boolean;
    last?: boolean;
    children: Snippet;
  }

  let { href, accent, clamp = true, last = false, children }: Props = $props();
</script>

<article
  class="surface tone-public has-accent"
  class:clickable={!!href}
  class:last-row={last}
  style={`--row-accent: ${accent};`}
>
  {#if href}
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href arrives resolved (ResolvedPathname) -->
    <a aria-label="Open item" class="surface-link" {href}></a>
  {/if}
  <div class="content" class:clamp-excerpts={clamp}>
    {@render children()}
  </div>
</article>
