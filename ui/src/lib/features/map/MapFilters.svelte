<script lang="ts">
  import IconMenuButton from '#lib/components/IconMenuButton.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import LocationPicker from '#lib/components/location/LocationPicker.svelte';
  import RadiusCombobox from '#lib/components/map/RadiusCombobox.svelte';
  import { radiusPresetOptions } from '#lib/domain/mapRadius.ts';
  import type { PlaceValue } from '#lib/domain/places.ts';
  import { placeSearch } from '#lib/services/locations.ts';
  import { filterOptions, windowOptions, type MapPanelState } from './panelState.svelte.ts';

  /** web's MapPanel filter row: under the map on desktop, over it on phones and fullscreen. */
  interface Props {
    s: MapPanelState;
    overlay: boolean;
    customDateOpen: boolean;
    onCustomDates: (button: HTMLButtonElement) => void;
    onPlace: (place: PlaceValue) => void;
    onLocate: () => void;
    onRadius: (value: string) => void;
    onFilters: () => void;
    onUpcoming: () => void;
    /** web's "Open as list": the Region feed on this place. */
    onList: () => void;
  }

  let {
    s,
    overlay,
    customDateOpen,
    onCustomDates,
    onPlace,
    onLocate,
    onRadius,
    onFilters,
    onUpcoming,
    onList,
  }: Props = $props();

  let dateButton = $state<HTMLButtonElement>();
</script>

{#snippet picker()}
  <LocationPicker
    value={s.place}
    onChange={onPlace}
    search={placeSearch.search}
    failure={placeSearch.failure}
    compact
    elevated
    preferAbove
    preserveCoordsWhileEditing
    portaled
    modes={['physical']}
    placeholder="Place"
  />
{/snippet}

{#if overlay && s.fullscreenSearchOpen}
  <div class="fullscreen-search">{@render picker()}</div>
{/if}

<div
  class={overlay ? 'filter-panel filter-panel-overlay' : 'filter-bar filter-bar-desktop'}
  aria-label="Map filters"
>
  {#if overlay}
    <button
      aria-expanded={s.fullscreenSearchOpen}
      aria-label="Search place"
      class="icon-button filter-icon-button"
      type="button"
      onclick={() => (s.fullscreenSearchOpen = !s.fullscreenSearchOpen)}
    >
      <ToolbarIcon name="search" />
    </button>
  {:else}
    <div class="place-field place-field-desktop">{@render picker()}</div>
  {/if}
  <button
    aria-label="Use my location"
    class={overlay ? 'icon-button filter-icon-button locate-overlay' : 'icon-button locate-desktop'}
    type="button"
    onclick={onLocate}
  >
    <ToolbarIcon name="locate" />
  </button>
  <RadiusCombobox
    value={s.radiusKm}
    onChange={onRadius}
    ariaLabel="Snap map to radius"
    options={radiusPresetOptions}
    compact={overlay}
    portaled
    preferAbove
  />
  <button
    aria-label="Upcoming only"
    title="Upcoming only"
    aria-pressed={s.upcomingOnly}
    class="icon-button filter-icon-button"
    class:active={s.upcomingOnly}
    type="button"
    onclick={onUpcoming}
  >
    <ToolbarIcon name={s.upcomingOnly ? 'calendar' : 'calendar-x'} />
  </button>
  <IconMenuButton
    value={s.windowFilter}
    ariaLabel="Map time window"
    defaultValue="today"
    options={windowOptions}
    onChange={(v) => {
      s.windowFilter = v;
      onFilters();
    }}
  >
    <ToolbarIcon name="clock" />
  </IconMenuButton>
  {#if s.showCustomDates}
    <div class="custom-date-wrap">
      <button
        bind:this={dateButton}
        aria-expanded={customDateOpen}
        aria-label="Custom date range"
        class="icon-button filter-icon-button"
        class:active={customDateOpen}
        type="button"
        onclick={(e) => {
          e.stopPropagation();
          if (dateButton) onCustomDates(dateButton);
        }}
      >
        <ToolbarIcon name="calendar-range" />
      </button>
    </div>
  {/if}
  <IconMenuButton
    value={s.entityFilter}
    ariaLabel="Map entity filter"
    defaultValue="all"
    options={filterOptions}
    onChange={(v) => {
      s.entityFilter = v;
      onFilters();
    }}
  >
    <ToolbarIcon name="filter" />
  </IconMenuButton>
  <button
    aria-label="Open as list"
    title="Open as list"
    class="icon-button filter-icon-button"
    disabled={!s.hasCenter}
    type="button"
    onclick={onList}
  >
    <ToolbarIcon name="list" />
  </button>
</div>
