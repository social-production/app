<script lang="ts">
  import type { MapPanelState } from './panelState.svelte.ts';

  /** Map Redesign §4 (owner, 2026-11-17): while pins load, the app's green line runs along the
   * top of the map and faint pin shapes pulse; when they land, "12 places" shows for a moment. */
  let { s }: { s: MapPanelState } = $props();

  let count = $state<number | null>(null);
  let wasLoading = false;

  $effect(() => {
    const loading = s.loading;
    if (loading) {
      wasLoading = true;
      count = null;
      return;
    }
    if (!wasLoading) return;
    wasLoading = false;
    count = s.listItems.length || null;
    const timer = setTimeout(() => (count = null), 2400);
    return () => clearTimeout(timer);
  });
</script>

{#if s.loading}
  <div class="map-loading" aria-hidden="true">
    <span class="map-loading-line"></span>
    <span class="map-ghost" style="left: 50%; top: 38%"></span>
    <span class="map-ghost" style="left: 30%; top: 55%"></span>
    <span class="map-ghost" style="left: 68%; top: 60%"></span>
  </div>
{:else if count}
  <span class="map-count-pill" role="status">{count === 1 ? '1 place' : `${count} places`}</span>
{/if}
