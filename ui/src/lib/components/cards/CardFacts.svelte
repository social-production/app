<script lang="ts">
  import { placeOnMap } from '#lib/components/map/placeOnMap.ts';
  import { implementedLabel } from '#lib/domain/eventSchedule.ts';

  /**
   * A feed card's when and where (owner, 2026-11-15): the date on its own line, then the
   * place on a separate soft row that opens the app's map on it. "Online" is not a place.
   */
  let { when = '', place = '' }: { when?: string; place?: string | null } = $props();
  const map = placeOnMap();

  const where = $derived(implementedLabel(place));
  const mappable = $derived(!!where && where.toLowerCase() !== 'online');

  function show(e: MouseEvent): void {
    if (!map || e.metaKey || e.ctrlKey || e.shiftKey) return;
    e.preventDefault();
    e.stopPropagation();
    map.show(where);
  }
</script>

{#if when || where}
  <div class="card-facts">
    {#if when}
      <p class="card-when">
        <svg aria-hidden="true" viewBox="0 0 24 24"
          ><rect x="4" y="5" width="16" height="15" rx="2" /><path d="M4 9h16M8 3v4M16 3v4" /></svg
        >
        {when}
      </p>
    {/if}
    {#if mappable}
      <a
        class="card-place"
        href={`https://www.openstreetmap.org/search?query=${encodeURIComponent(where)}`}
        rel="noopener noreferrer"
        title="Show on the map"
        onclick={show}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24"
          ><path d="M12 21s6.5-5.1 6.5-10a6.5 6.5 0 1 0-13 0c0 4.9 6.5 10 6.5 10Z" /><circle
            cx="12"
            cy="11"
            r="2.3"
          /></svg
        >
        <span>{where}</span>
        <svg class="card-place-go" aria-hidden="true" viewBox="0 0 24 24"
          ><path d="M9 4 3 6v14l6-2 6 2 6-2V4l-6 2zM9 4v14M15 6v14" /></svg
        >
      </a>
    {:else if where}
      <p class="card-when">{where}</p>
    {/if}
  </div>
{/if}
