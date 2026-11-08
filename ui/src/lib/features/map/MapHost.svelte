<script lang="ts">
  import type { Component } from 'svelte';

  /** Loads the map panel (and maplibre) only when it first opens, as web does (ADR 0085). */
  let { onClose, embedded = true }: { onClose: () => void; embedded?: boolean } = $props();

  let Panel = $state<Component<{ onClose: () => void; embedded?: boolean }>>();
  let failed = $state(false);

  $effect(() => {
    import('./MapPanel.svelte').then((m) => (Panel = m.default)).catch(() => (failed = true));
  });
</script>

{#if Panel}
  <Panel {onClose} {embedded} />
{:else if failed}
  <p class="inline-alert" role="alert">
    The map could not be loaded. Reload the page to try again.
  </p>
{/if}
