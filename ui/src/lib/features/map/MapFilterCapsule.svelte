<script lang="ts">
  import type { Snippet } from 'svelte';

  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import RadiusCombobox from '#lib/components/map/RadiusCombobox.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import SegmentedChoice from '#lib/components/SegmentedChoice.svelte';
  import { radiusPresetOptions } from '#lib/domain/mapRadius.ts';

  import { filterOptions, windowOptions, type MapPanelState } from './panelState.svelte.ts';

  /**
   * The map's filters on phones and in fullscreen (ADR 0103): one capsule over the top of the
   * map with a search button, the distance, Upcoming and Filters. Search opens a second bar
   * (place field and "my location") only when asked, so the map stays visible; Filters opens
   * a sheet with the time window and the type.
   */
  interface Props {
    s: MapPanelState;
    picker: Snippet;
    onLocate: () => void;
    onRadius: (value: string) => void;
    onFilters: () => void;
    onUpcoming: () => void;
    onList: () => void;
  }

  let { s, picker, onLocate, onRadius, onFilters, onUpcoming, onList }: Props = $props();

  let sheetOpen = $state(false);
  const narrowed = $derived(s.windowFilter !== 'all' || s.entityFilter !== 'all');
  const windows = windowOptions.filter((o) => o.value !== 'custom');
</script>

<div class="map-filter-top filter-panel-overlay" aria-label="Map filters">
  <div class="map-filter-capsule">
    <button
      aria-expanded={s.fullscreenSearchOpen}
      aria-label="Search place"
      title="Search place"
      class="capsule-icon"
      class:active={s.fullscreenSearchOpen}
      type="button"
      onclick={() => (s.fullscreenSearchOpen = !s.fullscreenSearchOpen)}
    >
      <ToolbarIcon name="search" />
    </button>
    <RadiusCombobox
      value={s.radiusKm}
      onChange={onRadius}
      ariaLabel="Snap map to radius"
      options={radiusPresetOptions}
      compact
      portaled
    />
    <button
      aria-pressed={s.upcomingOnly}
      title="Upcoming only"
      class="capsule-chip"
      class:active={s.upcomingOnly}
      type="button"
      onclick={onUpcoming}
    >
      <ToolbarIcon name={s.upcomingOnly ? 'calendar' : 'calendar-x'} /><span>Upcoming</span>
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
    {#if s.mapFullscreen}
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
    {/if}
  </div>
  {#if s.fullscreenSearchOpen}
    <div class="map-search-bar">
      <div class="map-search-field">{@render picker()}</div>
      <button
        aria-label="Use my location"
        title="Use my location"
        class="capsule-icon locate"
        type="button"
        onclick={onLocate}
      >
        <ToolbarIcon name="locate" />
      </button>
    </div>
  {/if}
</div>

<OverlaySheet
  open={sheetOpen}
  title="Map filters"
  labelledById="map-filter-title"
  rootClass="shell feed-filter-sheet map-filter-sheet"
  onClose={() => (sheetOpen = false)}
>
  <div class="feed-filter-groups">
    <section>
      <h3>When</h3>
      <SegmentedChoice
        value={s.windowFilter}
        ariaLabel="Map time window"
        options={windows}
        onChange={(v) => {
          s.windowFilter = v;
          onFilters();
        }}
      />
      <button
        class="custom-range"
        class:on={s.showCustomDates}
        aria-pressed={s.showCustomDates}
        type="button"
        onclick={() => {
          s.windowFilter = s.showCustomDates ? 'all' : 'custom';
          onFilters();
        }}>Custom range</button
      >
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
      <h3>Type</h3>
      <SegmentedChoice
        value={s.entityFilter}
        ariaLabel="Map entity filter"
        options={filterOptions}
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
        >Show results</button
      >
    </div>
  {/snippet}
</OverlaySheet>
