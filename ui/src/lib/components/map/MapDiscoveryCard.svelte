<script lang="ts">
  import type { ResolvedPathname } from '$app/types';
  import FeedSurface from '#lib/components/cards/FeedSurface.svelte';
  import SurfaceTypeLabel from '#lib/components/rail/SurfaceTypeLabel.svelte';
  import {
    markerFooter,
    markerMode,
    markerStatus,
    type MapMarkerItem,
  } from '#lib/domain/mapMarkers.ts';
  import { formatMarkerScheduleRange } from '#lib/domain/markerTime.ts';
  import { modeAccent } from '#lib/domain/projects.ts';
  import '#lib/components/styles/map.css';

  /** web's `components/cards/map/MapDiscoveryCard.svelte`: one result beside the map. */
  interface Props {
    item: MapMarkerItem;
    /** The item's page, resolved by the panel (with web's `?activity=` for an activity). */
    href: ResolvedPathname;
    isPast?: boolean;
    highlighted?: boolean;
    onHover: (id: string) => void;
    onLeave: () => void;
    onSelect: (id: string) => void;
    onClose: () => void;
  }

  let {
    item,
    href,
    isPast = false,
    highlighted = false,
    onHover,
    onLeave,
    onSelect,
    onClose,
  }: Props = $props();

  const mode = $derived(markerMode(item));
  const scheduleTime = $derived(formatMarkerScheduleRange(item.scheduledAt, item.endsAt));
  const status = $derived(markerStatus(item));
  const location = $derived(item.displayLabel.trim());
  const footer = $derived(markerFooter(item));

  function onclick(event: MouseEvent): void {
    onSelect(item.id);
    if ((event.target as HTMLElement | null)?.closest('a[href]')) onClose();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
<div
  class="map-discovery-card"
  class:past={isPast}
  class:highlighted
  {onclick}
  onmouseenter={() => onHover(item.id)}
  onmouseleave={onLeave}
>
  <FeedSurface {href} tone="public" accent={modeAccent(mode)}>
    <div class="header-row">
      <SurfaceTypeLabel projectMode={mode} />
    </div>
    {#if item.entityType === 'activity' && item.parentTitle?.trim()}
      <p class="parent-title" class:past={isPast}>{item.parentTitle}</p>
    {/if}
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href arrives resolved (ResolvedPathname) -->
    <a class="title" class:past={isPast} data-sveltekit-preload-data="hover" {href}>{item.title}</a>
    {#if status}
      <p class="detail-row status" class:past={isPast}>{status}</p>
    {/if}
    {#if scheduleTime}
      <p class="detail-row time" class:past={isPast}>{scheduleTime}</p>
    {/if}
    {#if location}
      <p class="detail-row location" class:past={isPast}>{location}</p>
    {/if}
    {#if footer}
      <p class="detail-row footer" class:past={isPast}>{footer}</p>
    {/if}
  </FeedSurface>
</div>
