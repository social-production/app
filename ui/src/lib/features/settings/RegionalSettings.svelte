<script lang="ts">
  import DistanceChoice from '#lib/components/settings/DistanceChoice.svelte';
  import PlaceSearchField from '#lib/components/settings/PlaceSearchField.svelte';
  import SettingPanel from '#lib/components/settings/SettingPanel.svelte';
  import SettingSwitch from '#lib/components/settings/SettingSwitch.svelte';
  import { devicePositionMessage } from '#lib/domain/defaultLocation.ts';
  import { radiusPresetOptions } from '#lib/domain/mapRadius.ts';
  import { emptyPlaceValue, type PlaceValue } from '#lib/domain/places.ts';
  import { devicePlace, ipHintFailure, ipPlaceHint, placeSearch } from '#lib/services/locations.ts';
  import { readDefaultLocation, writeDefaultLocation } from '#lib/stores/defaultLocation.svelte.ts';
  import {
    DEFAULT_REGION_RADIUS,
    readRegionRadius,
    writeRegionRadius,
  } from '#lib/stores/regionRadius.svelte.ts';

  /**
   * web's Regional settings: the default location (Region feed and map), the device-location
   * opt-in and "Set from network". Kept on this device (ADR 0081).
   */
  let { me }: { me: string } = $props();

  // svelte-ignore state_referenced_locally
  const saved = readDefaultLocation(me);
  let place = $state<PlaceValue>(saved?.place ?? emptyPlaceValue());
  let device = $state(saved?.deviceGeolocation ?? false);
  let message = $state('');
  let networkOpen = $state(false);
  // Owner's fix 3d: the place and the distance each open a small sheet with a Save button.
  let placeOpen = $state(false);
  let picked = $state<PlaceValue | null>(null);
  let distanceOpen = $state(false);
  let radius = $state(readRegionRadius());
  let draftRadius = $state(DEFAULT_REGION_RADIUS);
  const radiusLabel = (v: string): string =>
    radiusPresetOptions.find((o) => o.value === v)?.label ?? `${v} km`;
  $effect(() => {
    if (placeOpen) picked = null;
  });
  $effect(() => {
    if (distanceOpen) draftRadius = radius;
  });

  function save(next: PlaceValue): void {
    message = '';
    place = next;
    // web clears the default when the field is emptied.
    writeDefaultLocation(me, next.displayLabel.trim() ? next : emptyPlaceValue(), device);
  }

  async function fromNetwork(): Promise<void> {
    message = '';
    networkOpen = false;
    try {
      const [hint] = await ipPlaceHint();
      if (!hint || hint.latitude == null || hint.longitude == null) {
        message =
          'IP location is unavailable. On localhost use search or device location; on a deployed site this uses your public IP.';
        return;
      }
      save({ ...hint, mode: 'physical', isOnline: false });
    } catch (error) {
      message = ipHintFailure(error);
    }
  }

  async function locateOnce(): Promise<void> {
    const result = await devicePlace();
    if (result.ok) picked = result.place;
    else message = devicePositionMessage(result.error);
  }

  async function toggleDevice(): Promise<void> {
    message = '';
    device = !device;
    writeDefaultLocation(me, place, device);
    if (!device) return;
    const result = await devicePlace();
    if (!result.ok) {
      message = devicePositionMessage(result.error);
      device = false;
      writeDefaultLocation(me, place, false);
      return;
    }
    save(result.place);
  }
</script>

<section class="settings-section" id="settings-regional">
  <h2>Regional</h2>
  <div class="card stack flush">
    <SettingPanel
      label="Default location"
      value={place.displayLabel.trim() || 'Add'}
      hint="Used for the Region feed and the map."
      bind:open={placeOpen}
    >
      {#snippet footer()}
        <div class="sheet-actions">
          <button
            class="sheet-submit"
            type="button"
            disabled={!picked}
            onclick={() => {
              if (picked) save(picked);
              placeOpen = false;
            }}>{picked ? `Use ${picked.displayLabel.split(',')[0]}` : 'Pick a place'}</button
          >
        </div>
      {/snippet}
      {#if placeOpen}
        <PlaceSearchField
          initial={place.displayLabel}
          {picked}
          onPick={(p) => (picked = p)}
          onLocate={() => void locateOnce()}
          search={placeSearch.search}
          failure={placeSearch.failure}
        />
      {/if}
    </SettingPanel>
    <SettingPanel
      label="Distance"
      value={radiusLabel(radius)}
      hint="How far the Region feed looks from your place."
      bind:open={distanceOpen}
    >
      {#snippet footer()}
        <div class="sheet-actions">
          <button
            class="sheet-submit"
            type="button"
            onclick={() => {
              radius = draftRadius;
              writeRegionRadius(radius);
              distanceOpen = false;
            }}>Save</button
          >
        </div>
      {/snippet}
      <DistanceChoice
        value={draftRadius}
        presets={radiusPresetOptions}
        onChange={(v) => (draftRadius = v)}
      />
    </SettingPanel>
    <SettingSwitch
      label="Device location (GPS)"
      note="For the map’s “Use my location” button. Never used in the background."
      checked={device}
      onToggle={() => void toggleDevice()}
    />
    <SettingPanel
      label="Network location"
      value=""
      hint="A one-time set of your default place from your public IP, for when GPS isn’t available or you don’t want it. Coarser than GPS, and it only runs when you press the button."
      bind:open={networkOpen}
    >
      {#snippet footer()}
        <div class="sheet-actions">
          <button class="sheet-submit" type="button" onclick={() => void fromNetwork()}>
            Set from network
          </button>
        </div>
      {/snippet}
      <p class="setting-hint">{place.displayLabel.trim() || 'No default location yet.'}</p>
    </SettingPanel>
  </div>
  {#if message}<p class="status error" role="alert">{message}</p>{/if}
</section>
