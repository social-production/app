<script lang="ts">
  import { KIND_GLYPHS, type Kind } from '#lib/domain/kindGlyphs.ts';

  /** A kind's symbol (the map pins' and feed cards' shared set), in `currentColor`. */
  let { kind, size = 18 }: { kind: Kind; size?: number } = $props();
</script>

<svg aria-hidden="true" viewBox="0 0 24 24" width={size} height={size} class="kind-glyph">
  <!-- eslint-disable-next-line svelte/no-at-html-tags -- fixed paths from kindGlyphs.ts -->
  {@html KIND_GLYPHS[kind]}
</svg>
