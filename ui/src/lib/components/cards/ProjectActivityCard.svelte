<script lang="ts">
  import type { ResolvedPathname } from '$app/types';
  import FeedSurface from '#lib/components/cards/FeedSurface.svelte';
  import { isoOfMicros } from '#lib/domain/mapMarkers.ts';
  import { formatMarkerScheduleRange } from '#lib/domain/markerTime.ts';
  import { modeAccent, type CharterMode } from '#lib/domain/projects.ts';
  import type { RegionActivity } from '#lib/domain/regionFeed.ts';
  import '#lib/components/styles/cards.css';

  /** web's `ProjectActivityCard`: an upcoming project activity in the Region feed. The wrapper
   * carries the ported styles' root class (web's were scoped to the component). */
  let { item, href }: { item: RegionActivity; href: ResolvedPathname } = $props();

  const schedule = $derived(
    formatMarkerScheduleRange(isoOfMicros(item.scheduledAt), isoOfMicros(item.endsAt)),
  );
  const location = $derived(item.locationLabel.trim());
</script>

<div class="project-activity-card">
  <FeedSurface {href} tone="public" accent={modeAccent(item.projectMode as CharterMode)}>
    {#if item.parentTitle}
      <p class="parent-title">{item.parentTitle}</p>
    {/if}
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href arrives resolved (ResolvedPathname) -->
    <a class="title" data-sveltekit-preload-data="hover" {href}>{item.title}</a>
    {#if schedule}
      <p class="detail-row time">{schedule}</p>
    {/if}
    {#if location}
      <p class="detail-row location">{location}</p>
    {/if}
    <p class="detail-row footer">{item.distanceKm.toFixed(1)} km</p>
  </FeedSurface>
</div>
