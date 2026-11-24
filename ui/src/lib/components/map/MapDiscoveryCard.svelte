<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { ResolvedPathname } from '$app/types';
  import CardFacts from '#lib/components/cards/CardFacts.svelte';
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
  // A proposal's status is "Proposal", which its type already says (Map Redesign: "Kind · distance").
  const rawStatus = $derived(markerStatus(item));
  const status = $derived(rawStatus.toLowerCase() === mode ? '' : rawStatus);
  const location = $derived(item.displayLabel.trim());
  // web's footer is "1.2 km · 2/3 signed up": the distance goes up into the type line.
  const footerParts = $derived(markerFooter(item).split(' · '));
  const distance = $derived(footerParts[0] ?? '');
  const signups = $derived(footerParts.slice(1).join(' · '));
  const parent = $derived(item.entityType === 'activity' ? (item.parentTitle?.trim() ?? '') : '');

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
    <!-- Drawn as a Global feed row (owner, 2026-11-16): glyph, type · status · distance. -->
    <div class="feed-card-top feed-lead">
      <span class="feed-lead-glyph"
        ><SurfaceTypeLabel
          projectMode={mode}
          meetup={item.entityType === 'meetup'}
          feedGlyph
        /></span
      >
      <div class="feed-lead-copy">
        {#if parent}<div class="feed-lead-tags"><span class="map-parent">{parent}</span></div>{/if}
        <div class="feed-lead-meta" class:alone={!parent}>
          <SurfaceTypeLabel projectMode={mode} meetup={item.entityType === 'meetup'} />
          {#if status}<span class="feed-card-stage">{t(status)}</span>{/if}
          <span class="feed-card-time">{distance}</span>
        </div>
      </div>
    </div>
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href arrives resolved (ResolvedPathname) -->
    <a class="title" class:past={isPast} data-sveltekit-preload-data="hover" {href}>{item.title}</a>
    <CardFacts when={scheduleTime} place={location} />
    {#if signups}
      <p class="map-signups">{signups}</p>
    {/if}
  </FeedSurface>
</div>
