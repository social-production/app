<script lang="ts">
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import PillChoice from '#lib/components/PillChoice.svelte';
  import type { PlaceValue } from '#lib/domain/places.ts';
  import { shortPlace } from '#lib/domain/shortPlace.ts';

  import { filterOptions, windowOptions, type MapPanelState } from './panelState.svelte.ts';
  import PlaceSheet from './PlaceSheet.svelte';

  /**
   * The map's filters on phones and in fullscreen (Map Redesign §1–2, owner 2026-11-17): one
   * capsule at the bottom with "Place · distance" (one sheet for both), Filters (a dot when
   * anything is narrowed; Upcoming or Include past lives there now) and the list. "My
   * location" floats just above it on the right.
   */
  interface Props {
    s: MapPanelState;
    onPlace: (place: PlaceValue) => void;
    onLocate: () => void;
    onRadius: (value: string) => void;
    onFilters: () => void;
    onUpcoming: () => void;
    onList: () => void;
  }

  let { s, onPlace, onLocate, onRadius, onFilters, onUpcoming, onList }: Props = $props();

  let sheetOpen = $state(false);
  let placeOpen = $state(false);
  const narrowed = $derived(
    s.windowFilter !== 'all' || s.entityFilter !== 'all' || !s.upcomingOnly,
  );
  const distance = $derived(
    s.radiusKm === 'global'
      ? 'Global'
      : Number.isFinite(Number(s.radiusKm))
        ? `${Math.round(Number(s.radiusKm))} km`
        : s.radiusKm,
  );
  const where = $derived(shortPlace(s.place.displayLabel.trim()).split(',')[0] || 'Pick a place');
  const events = [
    { value: 'upcoming', label: 'Upcoming' },
    { value: 'past', label: 'Include past' },
  ];
  // Pills wrap whole words, so the range's own pill sits with the rest ("Pick dates").
  const windows = windowOptions.map((o) =>
    o.value === 'all'
      ? { ...o, label: 'Any time' }
      : o.value === 'custom'
        ? { ...o, label: 'Pick dates' }
        : o,
  );
  const types = filterOptions.map((o) =>
    o.value === 'help_requests'
      ? { ...o, label: 'Requests' }
      : o.value === 'all'
        ? { ...o, label: 'Everything' }
        : o,
  );
  const shown = $derived(s.listItems.length);
</script>

<div class="map-filter-top filter-panel-overlay" aria-label="Map filters">
  <button
    aria-label="Use my location"
    title="Use my location"
    class="map-locate-button"
    type="button"
    onclick={onLocate}
  >
    <ToolbarIcon name="locate" />
  </button>
  <div class="map-filter-capsule">
    <button
      aria-haspopup="dialog"
      aria-expanded={placeOpen}
      aria-label={`Search place. ${where}, ${distance}`}
      class="capsule-place"
      type="button"
      onclick={() => (placeOpen = true)}
    >
      <ToolbarIcon name="search" /><span>{where} · {distance}</span>
    </button>
    <button
      aria-haspopup="dialog"
      aria-expanded={sheetOpen}
      class="capsule-chip"
      class:narrowed
      type="button"
      onclick={() => (sheetOpen = true)}
    >
      <ToolbarIcon name="filter" /><span>Filters</span>
    </button>
    <button
      aria-label="Open as list"
      title="Open as list"
      class="capsule-icon"
      disabled={!s.hasCenter}
      type="button"
      onclick={onList}
    >
      <ToolbarIcon name="list" />
    </button>
  </div>
</div>

<PlaceSheet
  open={placeOpen}
  place={s.place}
  radius={s.radiusKm}
  onPlace={(p) => (onPlace(p), (placeOpen = false))}
  onLocate={() => (onLocate(), (placeOpen = false))}
  {onRadius}
  onClose={() => (placeOpen = false)}
/>

<OverlaySheet
  open={sheetOpen}
  title="Map filters"
  labelledById="map-filter-title"
  rootClass="shell feed-filter-sheet map-filter-sheet"
  onClose={() => (sheetOpen = false)}
>
  <div class="feed-filter-groups">
    <section>
      <h3>Events</h3>
      <PillChoice
        value={s.upcomingOnly ? 'upcoming' : 'past'}
        ariaLabel="Map events"
        options={events}
        onChange={(v) => {
          if ((v === 'upcoming') !== s.upcomingOnly) onUpcoming();
        }}
      />
    </section>
    <section>
      <h3>When</h3>
      <PillChoice
        value={s.windowFilter}
        ariaLabel="Map time window"
        options={windows}
        onChange={(v) => {
          s.windowFilter = v;
          onFilters();
        }}
      />
      {#if s.showCustomDates}
        <div class="custom-dates" role="group" aria-label="Custom date range">
          <label>
            <span>From</span>
            <input class="date-input" type="date" bind:value={s.dateFrom} onchange={onFilters} />
          </label>
          <label>
            <span>To</span>
            <input class="date-input" type="date" bind:value={s.dateTo} onchange={onFilters} />
          </label>
        </div>
      {/if}
    </section>
    <section>
      <h3>Show</h3>
      <PillChoice
        value={s.entityFilter}
        ariaLabel="Map entity filter"
        options={types}
        onChange={(v) => {
          s.entityFilter = v;
          onFilters();
        }}
      />
    </section>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" onclick={() => (sheetOpen = false)}
        >{shown === 1 ? 'Show 1 place' : `Show ${shown} places`}</button
      >
    </div>
  {/snippet}
</OverlaySheet>
