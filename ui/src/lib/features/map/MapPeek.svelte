<script lang="ts">
  import type { ResolvedPathname } from '$app/types';
  import KindGlyph from '#lib/components/icons/KindGlyph.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { KIND_LABELS, kindColor, kindOfMarker } from '#lib/domain/kindGlyphs.ts';
  import type { MapMarkerItem } from '#lib/domain/mapMarkers.ts';
  import { formatMarkerScheduleRange } from '#lib/domain/markerTime.ts';

  /** A tapped pin's card (Map Redesign, owner 2026-11-17): what it is, when and how far, with
   * Directions and Open; the map stays where it is until you choose. */
  interface Props {
    item: MapMarkerItem;
    href: ResolvedPathname;
    onClose: () => void;
    /** Opening the page leaves the map (its overlay closes). */
    onOpen: () => void;
  }

  let { item, href, onClose, onOpen }: Props = $props();

  const kind = $derived(kindOfMarker(item));
  const when = $derived(formatMarkerScheduleRange(item.scheduledAt, item.endsAt));
  const distance = $derived(
    item.distanceKm < 1
      ? `${Math.round(item.distanceKm * 1000)} m away`
      : `${item.distanceKm.toFixed(1)} km away`,
  );
  const line = $derived([KIND_LABELS[kind], when, distance].filter(Boolean).join(' · '));
  const directions = $derived(
    `https://www.openstreetmap.org/directions?to=${item.latitude}%2C${item.longitude}`,
  );
</script>

<div class="map-peek" role="dialog" aria-label={item.title} style={`--pin: ${kindColor(kind)}`}>
  <div class="map-peek-head">
    <span class="map-peek-icon"><KindGlyph {kind} size={18} /></span>
    <div>
      <b>{item.title}</b>
      <small>{line}</small>
    </div>
    <button class="map-peek-close" type="button" aria-label="Close" onclick={onClose}>
      <ToolbarIcon name="x" />
    </button>
  </div>
  <div class="map-peek-actions">
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- an outside site -->
    <a class="map-peek-secondary" href={directions} target="_blank" rel="noopener noreferrer"
      >Directions</a
    >
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by markerHref -->
    <a class="map-peek-primary" {href} onclick={onOpen}>Open</a>
  </div>
</div>
