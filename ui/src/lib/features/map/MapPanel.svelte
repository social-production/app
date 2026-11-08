<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import { goto } from '$app/navigation';
  import { portal } from '#lib/components/actions/portal.ts';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { effectiveRadiusKm } from '#lib/domain/mapRadius.ts';
  import type { PlaceValue } from '#lib/domain/places.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';
  import { clearMapFocusRequest, mapFocus } from '#lib/stores/mapFocus.svelte.ts';
  import { readMapPreferences } from '#lib/stores/mapPreferences.svelte.ts';
  import MapFilters from './MapFilters.svelte';
  import MapResults from './MapResults.svelte';
  import { MarkerLoader } from './markerLoader.ts';
  import { MapPanelState } from './panelState.svelte.ts';
  import { MapView } from './mapView.ts';
  import '#lib/components/styles/map.css';

  /** web's `features/map/MapPanel.svelte` (ADR 0085): the shell's map overlay and `/map`. */
  let { embedded = true, onClose }: { embedded?: boolean; onClose: () => void } = $props();

  const s = new MapPanelState();
  let mapContainer = $state<HTMLDivElement>();
  let customDateOpen = $state(false);
  let dateMenuStyle = $state('');
  let dateMenu = $state<HTMLDivElement>();
  let initialLoadDone = $state(false);

  // Pins and cards move inside the app (a page load would reconnect to Holochain).
  // eslint-disable-next-line svelte/no-navigation-without-resolve -- hrefs come from the index's own routes
  const navigate = (href: string): void => void goto(href).then(onClose);
  const view = new MapView(s, navigate, (coords) => loader.schedule(coords));
  const loader = new MarkerLoader(s, view, () => mapContainer);

  const stageStyle = $derived(
    s.isMobile && !s.mapFullscreen && s.mapStageHeightPx > 0
      ? `height: ${s.mapStageHeightPx}px; flex: 0 0 auto;`
      : '',
  );

  function changed(): void {
    view.resetViewportQuery();
    s.persistFilters();
    loader.schedule();
  }

  function onRadius(value: string): void {
    s.radiusKm = value;
    view.currentRadiusKm = effectiveRadiusKm(value);
    view.resetViewportQuery();
    s.persistFilters();
    void view.syncRadiusToCenter().then(() => loader.schedule());
  }

  async function toggleFullscreen(): Promise<void> {
    s.mapFullscreen = !s.mapFullscreen;
    customDateOpen = false;
    s.fullscreenSearchOpen = false;
    view.pauseViewportLoads(900);
    s.updateStageHeight();
    await tick();
    view.adapter?.resize?.();
  }

  async function openDates(button: HTMLButtonElement): Promise<void> {
    customDateOpen = !customDateOpen;
    if (!customDateOpen) return;
    await tick();
    const rect = button.getBoundingClientRect();
    const box = dateMenu?.getBoundingClientRect();
    if (!box) return;
    const top = Math.max(8, rect.top - box.height - 6);
    const left = Math.max(8, Math.min(rect.right - box.width, window.innerWidth - box.width - 8));
    dateMenuStyle = `top: ${top}px; left: ${left}px;`;
  }

  function onStageTransition(event: TransitionEvent, start: boolean): void {
    if (event.target !== event.currentTarget || event.propertyName !== 'height') return;
    view.stageTransitioning = start;
    if (start) view.pauseViewportLoads(500);
    view.scheduleResize(start ? 120 : 50);
  }

  async function start(): Promise<void> {
    const prefs = readMapPreferences(s.me);
    if (prefs) {
      s.radiusKm = prefs.radiusKm;
      s.windowFilter = prefs.windowFilter || 'all';
      s.entityFilter = prefs.entityFilter || 'all';
      s.upcomingOnly = prefs.upcomingOnly;
    }
    await loader.initialCenter();
    await tick();
    const { centerLat: lat, centerLon: lon } = s;
    if (lat != null && lon != null) {
      await view.mountIfNeeded(mapContainer, { lat, lon });
      await view.syncRadiusToCenter();
      view.currentRadiusKm = effectiveRadiusKm(s.radiusKm);
      view.syncRadiusOverlay();
      await loader.loadBroadening({ lat, lon });
    } else await view.mountIfNeeded(mapContainer);
    s.updateStageHeight();
    await tick();
    view.adapter?.resize?.();
    initialLoadDone = true;
  }

  // Start once the connection settles, so a signed-in person's saved place and filters are
  // read under their key (straight after a page load the connection is still opening).
  let started = false;
  $effect(() => {
    const current = connection.current;
    if (started || current.status === 'connecting') return;
    started = true;
    s.me = current.status === 'connected' ? current.agent : null;
    untrack(() => void start());
  });

  onMount(() => {
    const phone = window.matchMedia('(max-width: 760px)');
    const narrow = window.matchMedia('(max-width: 900px)');
    const sync = (): void => {
      s.isMobile = phone.matches;
      s.isCompactMap = narrow.matches;
      s.updateStageHeight();
      view.scheduleResize();
    };
    const closeDates = (e: MouseEvent): void => {
      if (customDateOpen && !dateMenu?.contains(e.target as Node)) customDateOpen = false;
    };
    sync();
    phone.addEventListener('change', sync);
    narrow.addEventListener('change', sync);
    window.addEventListener('resize', sync);
    document.addEventListener('click', closeDates);
    return () => {
      phone.removeEventListener('change', sync);
      narrow.removeEventListener('change', sync);
      window.removeEventListener('resize', sync);
      document.removeEventListener('click', closeDates);
      loader.destroy();
      view.destroy();
      s.destroy();
    };
  });

  $effect(() => {
    const request = mapFocus.request;
    if (!initialLoadDone || !request) return;
    untrack(() => {
      clearMapFocusRequest();
      // web shows an error toast; the app has no toast host, so the map's own toast says it.
      void loader.focus(request, (label) => s.showToast(`Could not find ${label} on the map`));
    });
  });
</script>

<section class="map-panel" class:embedded class:map-panel-fullscreen={s.mapFullscreen}>
  {#if s.mapTileError}
    <p class="inline-alert" role="alert">{s.mapTileError}</p>
  {/if}

  <div class="layout" class:layout-fullscreen={s.mapFullscreen}>
    <div class="map-column">
      <div
        class="map-stage"
        class:map-stage-fullscreen={s.mapFullscreen}
        class:map-stage-animated={s.isMobile && !s.mapFullscreen && s.mapStageHeightPx > 0}
        style={stageStyle}
        ontransitionstart={(e) => onStageTransition(e, true)}
        ontransitionend={(e) => onStageTransition(e, false)}
      >
        <div class="map-surface" bind:this={mapContainer} role="presentation"></div>
        {#if s.locationToast}
          <div class="location-toast" role="alert">{s.locationToast}</div>
        {/if}
        <button
          aria-label={s.mapFullscreen ? 'Exit fullscreen map' : 'Fullscreen map'}
          class="fullscreen-button"
          type="button"
          onclick={() => void toggleFullscreen()}
        >
          <ToolbarIcon name={s.mapFullscreen ? 'compress' : 'expand'} />
        </button>
        {#if s.overlayFilters}
          {@render filters(true)}
        {/if}
      </div>
      {#if !s.mapFullscreen && !s.isCompactMap}
        {@render filters(false)}
      {/if}
    </div>

    {#if !s.mapFullscreen}
      <MapResults
        {s}
        onHover={(id) => !s.isMobile && view.highlight(id)}
        onLeave={() => !s.isMobile && view.highlight(null)}
        onSelect={(id) => view.highlight(id, true)}
        {onClose}
      />
    {/if}
  </div>
</section>

{#snippet filters(overlay: boolean)}
  <MapFilters
    {s}
    {overlay}
    {customDateOpen}
    onCustomDates={(b) => void openDates(b)}
    onPlace={(place: PlaceValue) => void loader.applyAnchor(place, { broadenEmpty: true })}
    onLocate={() => void loader.useDevice()}
    {onRadius}
    onFilters={changed}
    onUpcoming={() => {
      s.upcomingOnly = !s.upcomingOnly;
      changed();
    }}
  />
{/snippet}

{#if customDateOpen}
  <div class="map-panel map-portal" use:portal>
    <div
      bind:this={dateMenu}
      class="custom-date-menu custom-date-menu-portaled"
      role="group"
      aria-label="Custom date range"
      style={dateMenuStyle}
    >
      <label class="custom-date-field">
        <span>From</span>
        <input class="date-input" type="date" bind:value={s.dateFrom} onchange={changed} />
      </label>
      <label class="custom-date-field">
        <span>To</span>
        <input class="date-input" type="date" bind:value={s.dateTo} onchange={changed} />
      </label>
    </div>
  </div>
{/if}
