<script lang="ts">
  import { locale, t } from '#lib/i18n/index.ts';
  import type { ResolvedPathname } from '$app/types';
  import KindGlyph from '#lib/components/icons/KindGlyph.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { KIND_LABELS, kindColor, kindOfMarker } from '#lib/domain/kindGlyphs.ts';
  import type { MapMarkerItem } from '#lib/domain/mapMarkers.ts';
  import type { RegionCard } from '#lib/domain/regionFeed.ts';
  import { SPACE_PICTURES, type SpacePictureOf } from '#lib/domain/scopes.ts';
  import { getContext } from 'svelte';
  import { formatMarkerScheduleRange } from '#lib/domain/markerTime.ts';
  import { shortPlace } from '#lib/domain/shortPlace.ts';
  import { wall } from '#lib/domain/zone.ts';
  import DirectionsSheet from './DirectionsSheet.svelte';

  /**
   * A tapped pin's card (Map cards mockup v2, owner 2026-11-25): the meetup, event or project
   * with its picture, type and distance, then its next sessions (up to three); Open goes to
   * the first one. A request's place is approximate: "about here" and no Directions.
   */
  interface Props {
    /** What the card is about: the parent, or a lone item. */
    item: MapMarkerItem;
    /** Its feed card: picture, type, channel. */
    card?: RegionCard | null;
    sessions: MapMarkerItem[];
    href: ResolvedPathname;
    hrefOf: (item: MapMarkerItem) => ResolvedPathname;
    onClose: () => void;
    /** Opening the page leaves the map (its overlay closes). */
    onOpen: () => void;
  }

  let { item, card = null, sessions, href, hrefOf, onClose, onOpen }: Props = $props();

  // A meetup's session is a meetup's, not an event's (owner, 2026-11-25).
  const kind = $derived(card?.subtype === 'meetup' ? 'meetup' : kindOfMarker(item));
  const pictureOf = getContext<SpacePictureOf | undefined>(SPACE_PICTURES);
  const picture = $derived(card?.picture ? (pictureOf?.(card.picture) ?? null) : null);
  const approximate = $derived(
    item.precision === 'approximate' && item.entityType === 'help_request',
  );
  const title = $derived(
    card?.title ?? (item.entityType === 'activity' ? item.parentTitle : null) ?? item.title,
  );
  const channel = $derived(card?.tags?.[0]?.replace(/^[#@]/, '') ?? '');
  const distance = $derived(
    item.distanceKm < 1
      ? `${Math.round(item.distanceKm * 1000)} m`
      : `${item.distanceKm.toFixed(1)} km`,
  );
  const line = $derived(
    [t(KIND_LABELS[kind]), channel, approximate ? t('about here') : distance]
      .filter(Boolean)
      .join(' · '),
  );
  const shown = $derived(sessions.slice(0, 3));
  const openHref = $derived(sessions[0] ? hrefOf(sessions[0]) : href);
  let directionsOpen = $state(false);
  const to = $derived({
    latitude: item.latitude,
    longitude: item.longitude,
    label: item.displayLabel.trim() || item.title,
  });

  // The app's chosen time zone, as everywhere else.
  const at = (iso: string | null): Date | null => (iso ? wall(Date.parse(iso)) : null);
  const day = (iso: string | null, opts: Intl.DateTimeFormatOptions): string =>
    at(iso)?.toLocaleDateString(locale(), opts) ?? '';
  const clock = (iso: string | null): string =>
    at(iso)?.toLocaleTimeString(locale(), { hour: 'numeric', minute: '2-digit' }) ?? '';
  const times = (s: MapMarkerItem): string =>
    [clock(s.scheduledAt), clock(s.endsAt)].filter(Boolean).join(' – ');
  // "0 of 6" (going of the minimum), as the mockup; "8 going" once it's met.
  const going = (s: MapMarkerItem): string => {
    if (s.minimumParticipants == null) return '';
    const n = s.committedCount ?? 0;
    return n < s.minimumParticipants
      ? t('{n} of {min}', { n, min: s.minimumParticipants })
      : t('{n} going', { n });
  };
  // An event with no sessions gets its own date row, like a session's (owner, 2026-11-25:
  // the date crammed into the type line looked messy).
  const own = $derived(!sessions.length && !!item.scheduledAt);
  const sameDay = $derived(
    !item.endsAt ||
      day(item.scheduledAt, { dateStyle: 'short' }) === day(item.endsAt, { dateStyle: 'short' }),
  );
  const ownDate = $derived(
    sameDay
      ? day(item.scheduledAt, { weekday: 'long', day: 'numeric', month: 'long' })
      : formatMarkerScheduleRange(item.scheduledAt, item.endsAt),
  );
  const ownDetail = $derived(
    [sameDay ? times(item) : '', shortPlace(item.displayLabel)].filter(Boolean).join(' · '),
  );
</script>

<div class="map-peek" role="dialog" aria-label={title} style={`--pin: ${kindColor(kind)}`}>
  <div class="map-peek-head">
    <span class="map-peek-icon" class:has-picture={!!picture}
      >{#if picture}<img src={picture} alt="" />{:else}<KindGlyph {kind} size={18} />{/if}</span
    >
    <div>
      <b>{title}</b>
      <small>{line}</small>
    </div>
    <button class="map-peek-close" type="button" aria-label={t('Close')} onclick={onClose}>
      <ToolbarIcon name="x" />
    </button>
  </div>
  {#if own}
    <div class="map-peek-sessions">
      <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by markerHref -->
      <a class="map-peek-session" {href} onclick={onOpen}>
        <span class="map-peek-day"
          ><small>{day(item.scheduledAt, { weekday: 'short' })}</small><b
            >{day(item.scheduledAt, { day: 'numeric' })}</b
          ></span
        >
        <span class="map-peek-session-copy">
          <b>{ownDate}</b>
          {#if ownDetail}<span>{ownDetail}</span>{/if}
        </span>
        {#if going(item)}<em>{going(item)}</em>{/if}
      </a>
    </div>
  {:else if shown.length > 0}
    <div class="map-peek-sessions">
      {#each shown as s (s.id)}
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by markerHref -->
        <a class="map-peek-session" href={hrefOf(s)} onclick={onOpen}>
          <span class="map-peek-day"
            ><small>{day(s.scheduledAt, { weekday: 'short' })}</small><b
              >{day(s.scheduledAt, { day: 'numeric' })}</b
            ></span
          >
          <span class="map-peek-session-copy">
            <b>{s.title}</b>
            <span>{[times(s), shortPlace(s.displayLabel)].filter(Boolean).join(' · ')}</span>
          </span>
          {#if going(s)}<em>{going(s)}</em>{/if}
        </a>
      {/each}
      {#if sessions.length > shown.length}
        <p class="map-peek-more">{t('+{n} more', { n: sessions.length - shown.length })}</p>
      {/if}
    </div>
  {/if}
  <div class="map-peek-actions">
    {#if !approximate}
      <button class="map-peek-secondary" type="button" onclick={() => (directionsOpen = true)}
        >{t('Directions')}</button
      >
    {/if}
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by markerHref -->
    <a class="map-peek-primary" href={openHref} onclick={onOpen}>{t('Open')}</a>
  </div>
</div>
<DirectionsSheet open={directionsOpen} {to} onClose={() => (directionsOpen = false)} />
