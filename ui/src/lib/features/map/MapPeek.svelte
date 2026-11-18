<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { ResolvedPathname } from '$app/types';
  import KindGlyph from '#lib/components/icons/KindGlyph.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { KIND_LABELS, kindColor, kindOfMarker } from '#lib/domain/kindGlyphs.ts';
  import type { MapMarkerItem } from '#lib/domain/mapMarkers.ts';
  import { formatMarkerScheduleRange } from '#lib/domain/markerTime.ts';
  import DirectionsSheet from './DirectionsSheet.svelte';

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
  let directionsOpen = $state(false);
  const to = $derived({
    latitude: item.latitude,
    longitude: item.longitude,
    label: item.displayLabel.trim() || item.title,
  });
</script>

<div class="map-peek" role="dialog" aria-label={item.title} style={`--pin: ${kindColor(kind)}`}>
  <div class="map-peek-head">
    <span class="map-peek-icon"><KindGlyph {kind} size={18} /></span>
    <div>
      <b>{item.title}</b>
      <small>{line}</small>
    </div>
    <button class="map-peek-close" type="button" aria-label={t('Close')} onclick={onClose}>
      <ToolbarIcon name="x" />
    </button>
  </div>
  <div class="map-peek-actions">
    <button class="map-peek-secondary" type="button" onclick={() => (directionsOpen = true)}
      >{t('Directions')}</button
    >
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by markerHref -->
    <a class="map-peek-primary" {href} onclick={onOpen}>{t('Open')}</a>
  </div>
</div>

<DirectionsSheet open={directionsOpen} {to} onClose={() => (directionsOpen = false)} />
