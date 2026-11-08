<script lang="ts">
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import RadiusCombobox from '#lib/components/map/RadiusCombobox.svelte';
  import { radiusPresetOptions } from '#lib/domain/mapRadius.ts';

  import type { RegionFeedState } from './regionState.svelte.ts';

  /** web's PublicFeed Region controls: radius, place search, device location, online list. */
  let { region }: { region: RegionFeedState } = $props();

  let focused = $state(false);
  let blurTimer: ReturnType<typeof setTimeout> | null = null;
  const expanded = $derived(focused || region.suggestions.length > 0);

  function blur(): void {
    blurTimer = setTimeout(() => {
      focused = false;
      region.suggestions = [];
    }, 180);
  }
</script>

<RadiusCombobox
  value={region.radius}
  onChange={(v) => {
    region.radius = v;
    void region.reload();
  }}
  ariaLabel="Region radius"
  compact
  options={radiusPresetOptions}
/>

<label class="place-search" class:expanded>
  <input
    aria-label="Place"
    value={region.placeQuery}
    placeholder="Place"
    onfocus={() => {
      if (blurTimer) clearTimeout(blurTimer);
      focused = true;
    }}
    onblur={blur}
    oninput={(e) => region.search(e.currentTarget.value)}
  />
</label>

<button
  aria-label="Use my location"
  class="icon-button"
  type="button"
  onclick={() => void region.useDevice()}
>
  <ToolbarIcon name="locate" />
</button>

<button
  aria-label="Include online list"
  aria-pressed={region.includeOnline}
  class="icon-button region-primary-only"
  class:active={region.includeOnline}
  title="Shows online events separately from the map"
  type="button"
  onclick={() => {
    region.includeOnline = !region.includeOnline;
    void region.reload();
  }}
>
  <ToolbarIcon name="wifi" />
</button>
