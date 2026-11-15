<script lang="ts">
  import type { ResolvedPathname } from '$app/types';
  import { resolve } from '$app/paths';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import MapDiscoveryCard from '#lib/components/map/MapDiscoveryCard.svelte';
  import { markerRoute, type MapMarkerItem } from '#lib/domain/mapMarkers.ts';
  import type { MapPanelState } from './panelState.svelte.ts';

  /** web's MapPanel results list beside (or under) the map. */
  interface Props {
    s: MapPanelState;
    onHover: (id: string) => void;
    onLeave: () => void;
    onSelect: (id: string) => void;
    onClose: () => void;
    /** web's "Open as list", in the results' header on phones (ADR 0103). */
    onList: () => void;
  }

  let { s, onHover, onLeave, onSelect, onClose, onList }: Props = $props();

  /** The item's page, with web's `?activity=` (the path is resolved; the query rides along). */
  function hrefOf(item: MapMarkerItem): ResolvedPathname {
    const route = markerRoute(item.href);
    if (!route) return resolve('/');
    const path =
      route.path === 'events'
        ? resolve('/events/[id]', { id: route.id })
        : route.path === 'help-requests'
          ? resolve('/help-requests/[id]', { id: route.id })
          : resolve('/projects/[id]', { id: route.id });
    return `${path}${route.query}` as ResolvedPathname;
  }

  const isPast = (iso: string | null): boolean => !!iso && new Date(iso).getTime() < Date.now();
</script>

<section class="list-panel" aria-label="Map results list">
  {#if s.loading && s.listItems.length === 0}
    <p class="empty-state">Loading markers…</p>
  {:else if s.errorMessage}
    <p class="empty-state" role="alert">{s.errorMessage}</p>
  {:else if !s.hasCenter}
    <p class="empty-state">
      Search a place, use device location, or set a default in Settings to load map pins.
    </p>
  {:else if s.listItems.length === 0}
    <p class="empty-state">No physical map-eligible items in this radius for the selected time.</p>
  {:else}
    {#if s.overlayFilters}
      <header class="results-head">
        <strong>{s.listItems.length} near here</strong>
        <button aria-label="Open as list" class="results-list" type="button" onclick={onList}>
          <ToolbarIcon name="list" /><span>List</span>
        </button>
      </header>
    {/if}
    <div class="card-list">
      {#each s.listItems as item (item.id)}
        <MapDiscoveryCard
          {item}
          href={hrefOf(item)}
          isPast={!s.upcomingOnly && isPast(item.scheduledAt)}
          highlighted={s.highlightedId === item.id}
          {onHover}
          {onLeave}
          {onSelect}
          {onClose}
        />
      {/each}
    </div>
  {/if}
</section>
