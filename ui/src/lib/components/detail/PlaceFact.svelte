<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { page } from '$app/state';
  import { placeOnMap } from '#lib/components/map/placeOnMap.ts';

  import FactRow from './FactRow.svelte';

  /** A detail page's place as a facts row that opens the app's map on it; "online" stays a
   * plain row (Detail Pages Cleanup §3). */
  let { place }: { place: string } = $props();
  const map = placeOnMap();

  const where = $derived(place.trim());
  const mappable = $derived(!!where && where.toLowerCase() !== 'online');

  function show(e: MouseEvent): void {
    if (!map || e.metaKey || e.ctrlKey || e.shiftKey) return;
    e.preventDefault();
    map.show(where, page.url.pathname);
  }
</script>

{#if mappable}
  <FactRow
    icon="where"
    title={where}
    sub={t('Show on the map')}
    href={`https://www.openstreetmap.org/search?query=${encodeURIComponent(where)}`}
    external
    onclick={show}
  />
{:else if where}
  <FactRow icon="where" title={where} />
{/if}
