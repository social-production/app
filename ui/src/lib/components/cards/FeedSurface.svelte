<script lang="ts">
  import type { Snippet } from 'svelte';

  import type { ResolvedPathname } from '$app/types';

  import '#lib/components/styles/cards.css';

  /** web's FeedSurface: one feed row; the whole row links to `href`. */
  interface Props {
    href: ResolvedPathname;
    /** CSS colour for the left accent bar (web's surface type accent). */
    accent: string;
    children: Snippet;
  }

  let { href, accent, children }: Props = $props();
</script>

<article class="surface tone-public clickable has-accent" style={`--row-accent: ${accent};`}>
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href arrives resolved (ResolvedPathname) -->
  <a aria-label="Open item" class="surface-link" {href}></a>
  <div class="content clamp-excerpts">
    {@render children()}
  </div>
</article>
