<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import RadiusChips from '#lib/components/map/RadiusChips.svelte';
  import { radiusPresetOptions } from '#lib/domain/mapRadius.ts';

  import type { RegionFeedState } from './regionState.svelte.ts';

  /**
   * web's PublicFeed Region controls (place search, device location, radius, online list), as
   * the top of the feed's filter sheet (Polish Pass mockups, 2026-11-16). The distance box takes
   * a typed distance as well as the presets (owner, 2026-11-16); they are chips, so picking one
   * never opens the keyboard (owner, 2026-11-17).
   */
  let { region }: { region: RegionFeedState } = $props();

  let focused = $state(false);
  let blurTimer: ReturnType<typeof setTimeout> | null = null;

  function blur(): void {
    blurTimer = setTimeout(() => {
      focused = false;
      region.suggestions = [];
    }, 180);
  }
</script>

<section>
  <h3>{t('Place')}</h3>
  <div class="region-place" class:focused>
    <ToolbarIcon name="map-pin" />
    <input
      aria-label={t('Place')}
      value={region.placeQuery}
      placeholder={t('Search a place')}
      onfocus={() => {
        if (blurTimer) clearTimeout(blurTimer);
        focused = true;
      }}
      onblur={blur}
      oninput={(e) => region.search(e.currentTarget.value)}
      onkeydown={(e) => {
        const first = region.suggestions[0];
        if (e.key === 'Enter' && first) {
          e.preventDefault();
          void region.select(first);
        }
      }}
      autocomplete="off"
      enterkeyhint="search"
    />
    {#if region.searching}
      <span class="region-spinner" aria-label={t('Searching')}></span>
    {:else if focused && region.placeQuery}
      <button
        aria-label={t('Clear place')}
        class="region-clear"
        type="button"
        onmousedown={(e) => e.preventDefault()}
        onclick={() => region.search('')}
      >
        <ToolbarIcon name="x" />
      </button>
    {/if}
    <button
      aria-label={t('Use my location')}
      title={t('Use my location')}
      class="region-locate"
      type="button"
      onclick={() => void region.useDevice()}
    >
      <ToolbarIcon name="locate" />
    </button>
  </div>
  {#if region.message}
    <p class="region-note" role="alert">
      <ToolbarIcon name="locate" /><span>{region.message}</span>
    </p>
  {/if}
  {#if region.suggestions.length > 0}
    <ul class="region-suggestions">
      {#each region.suggestions as place, i (i)}
        <li>
          <button
            type="button"
            onmousedown={(e) => e.preventDefault()}
            onclick={() => void region.select(place)}>{t(place.displayLabel)}</button
          >
        </li>
      {/each}
    </ul>
  {/if}
</section>

<section>
  <h3>{t('Distance')}</h3>
  <RadiusChips
    value={region.radius}
    onChange={(v) => {
      region.radius = v;
      void region.reload();
    }}
    ariaLabel={t('Region radius')}
    options={radiusPresetOptions}
  />
</section>

<div class="region-online">
  <span>{t('Include online events')}</span>
  <button
    aria-checked={region.includeOnline}
    aria-label={t('Include online events')}
    class="soft-switch"
    class:on={region.includeOnline}
    role="switch"
    type="button"
    onclick={() => {
      region.includeOnline = !region.includeOnline;
      void region.reload();
    }}
  >
    <span></span>
  </button>
</div>
