<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import LocationPicker from '#lib/components/location/LocationPicker.svelte';
  import RadiusChips from '#lib/components/map/RadiusChips.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { radiusPresetOptions } from '#lib/domain/mapRadius.ts';
  import type { PlaceValue } from '#lib/domain/places.ts';
  import { placeSearch } from '#lib/services/locations.ts';

  /** Map Redesign §1 (owner, 2026-11-17): the capsule's "Place · distance" opens one sheet for
   * both. The map stays in view behind it under a light tint and follows each choice. */
  interface Props {
    open: boolean;
    place: PlaceValue;
    radius: string;
    onPlace: (place: PlaceValue) => void;
    onLocate: () => void;
    onRadius: (value: string) => void;
    onClose: () => void;
  }

  let { open, place, radius, onPlace, onLocate, onRadius, onClose }: Props = $props();
</script>

<OverlaySheet
  {open}
  title={t('Place and distance')}
  labelledById="map-place-title"
  rootClass="shell feed-filter-sheet radius-sheet place-sheet"
  {onClose}
>
  <div class="feed-filter-groups">
    <section>
      <h3>{t('Place')}</h3>
      <LocationPicker
        value={place}
        onChange={onPlace}
        search={placeSearch.search}
        failure={placeSearch.failure}
        compact
        preserveCoordsWhileEditing
        modes={['physical']}
        placeholder={t('Search a place')}
      />
      <button class="place-locate" type="button" onclick={onLocate}>
        <ToolbarIcon name="locate" /><span>{t('Use my location')}</span>
      </button>
    </section>
    <section>
      <h3>{t('Distance')}</h3>
      <RadiusChips
        value={radius}
        options={radiusPresetOptions}
        ariaLabel={t('Distance')}
        onChange={onRadius}
      />
    </section>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" onclick={onClose}>{t('Done')}</button>
    </div>
  {/snippet}
</OverlaySheet>
