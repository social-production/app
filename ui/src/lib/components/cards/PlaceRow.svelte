<script lang="ts">
  import { openPlaceOnMap } from '#lib/stores/mapFocus.svelte.ts';

  import FactLine from './FactLine.svelte';

  /** A detail page's place (Plan, Phase and Feedback Screens §1): a soft row that opens the
   * app's map on it; "online" stays a plain line. */
  let { place }: { place: string } = $props();

  const where = $derived(place.trim());
  const mappable = $derived(!!where && where.toLowerCase() !== 'online');

  function show(e: MouseEvent): void {
    if (e.metaKey || e.ctrlKey || e.shiftKey) return;
    e.preventDefault();
    openPlaceOnMap(where);
  }
</script>

{#if mappable}
  <a
    class="detail-place"
    href={`https://www.openstreetmap.org/search?query=${encodeURIComponent(where)}`}
    rel="noopener noreferrer"
    onclick={show}
  >
    <FactLine icon="where"><strong>{where}<small>Show on the map</small></strong></FactLine>
    <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M9.5 6.5 15 12l-5.5 5.5" /></svg>
  </a>
{:else if where}
  <FactLine icon="where">{where}</FactLine>
{/if}
