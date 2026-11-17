<script lang="ts">
  import type { Snippet } from 'svelte';

  import '#lib/components/styles/ux/detailfacts.css';

  /** A detail page's facts card: its `FactRow`s, grouped (Detail Pages Cleanup §3). */
  let { children }: { children: Snippet } = $props();
</script>

<div class="detail-facts">{@render children()}</div>
