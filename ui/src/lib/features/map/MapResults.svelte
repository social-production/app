<script lang="ts">
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import MapDiscoveryCard from '#lib/components/map/MapDiscoveryCard.svelte';
  import { markerHref } from './markerHref.ts';
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

  const WHEN: Record<string, string> = {
    today: ' today',
    week: ' this week',
    month: ' this month',
    custom: ' in those dates',
  };
  // The empty map says what was searched and what to change (Scope Filters' empty states).
  const nothingText = $derived.by(() => {
    const where = s.radiusKm === 'global' ? 'anywhere' : `within ${s.radiusKm} km`;
    const when = WHEN[s.windowFilter] ?? '';
    const wider = s.radiusKm === 'global' ? '' : 'a wider distance';
    const longer = when ? 'a longer time' : '';
    const tryThis = [wider, longer].filter(Boolean).join(' or ');
    return `Nothing on the map ${where}${when}.${tryThis ? ` Try ${tryThis}.` : ''}`;
  });

  const isPast = (iso: string | null): boolean => !!iso && new Date(iso).getTime() < Date.now();
</script>

<section class="list-panel" aria-label="Map results list">
  {#if s.loading && s.listItems.length === 0}
    <SkeletonRows count={3} kind="row" />
  {:else if s.errorMessage}
    <EmptyState icon="map-pin" title="The map couldn't load" text={s.errorMessage} />
  {:else if !s.hasCenter}
    <EmptyState
      icon="map-pin"
      title="Choose a place"
      text="Search a place or use your location to see what's near it."
    />
  {:else if s.listItems.length === 0}
    <EmptyState icon="map-pin" title="Nothing here yet" text={nothingText} />
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
          href={markerHref(item)}
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
