<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { ResolvedPathname } from '$app/types';
  import KindGlyph from '#lib/components/icons/KindGlyph.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { KIND_LABELS, kindColor, kindOfMarker } from '#lib/domain/kindGlyphs.ts';
  import type { MapMarkerItem } from '#lib/domain/mapMarkers.ts';
  import type { RegionCard } from '#lib/domain/regionFeed.ts';
  import { SPACE_PICTURES, type SpacePictureOf } from '#lib/domain/scopes.ts';
  import { getContext } from 'svelte';
  import { formatMarkerScheduleRange } from '#lib/domain/markerTime.ts';
  import DirectionsSheet from './DirectionsSheet.svelte';

  /** A tapped pin's card (Map Redesign, owner 2026-11-17): what it is, when and how far, with
   * Directions and Open; the map stays where it is until you choose. */
  interface Props {
    item: MapMarkerItem;
    /** The item's feed card (its parent's, for a session): picture and type. */
    card?: RegionCard | null;
    href: ResolvedPathname;
    onClose: () => void;
    /** Opening the page leaves the map (its overlay closes). */
    onOpen: () => void;
  }

  let { item, card = null, href, onClose, onOpen }: Props = $props();

  // A meetup's session is a meetup's, not an event's (owner, 2026-11-25).
  const kind = $derived(card?.subtype === 'meetup' ? 'meetup' : kindOfMarker(item));
  const pictureOf = getContext<SpacePictureOf | undefined>(SPACE_PICTURES);
  const picture = $derived(card?.picture ? (pictureOf?.(card.picture) ?? null) : null);
  const session = $derived(item.entityType === 'activity' && !!item.parentTitle);
  const when = $derived(formatMarkerScheduleRange(item.scheduledAt, item.endsAt));
  const distance = $derived(
    item.distanceKm < 1
      ? `${Math.round(item.distanceKm * 1000)} m away`
      : `${item.distanceKm.toFixed(1)} km away`,
  );
  const line = $derived(
    [
      session ? t('Session of {name}', { name: item.parentTitle ?? '' }) : t(KIND_LABELS[kind]),
      when,
      distance,
    ]
      .filter(Boolean)
      .join(' · '),
  );
  let directionsOpen = $state(false);
  const to = $derived({
    latitude: item.latitude,
    longitude: item.longitude,
    label: item.displayLabel.trim() || item.title,
  });
</script>

<div class="map-peek" role="dialog" aria-label={item.title} style={`--pin: ${kindColor(kind)}`}>
  <div class="map-peek-head">
    <span class="map-peek-icon" class:has-picture={!!picture}
      >{#if picture}<img src={picture} alt="" />{:else}<KindGlyph {kind} size={18} />{/if}</span
    >
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
