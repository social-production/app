<script lang="ts">
  import LocationPicker from '#lib/components/location/LocationPicker.svelte';
  import SettingPanel from '#lib/components/settings/SettingPanel.svelte';
  import SettingSwitch from '#lib/components/settings/SettingSwitch.svelte';
  import { devicePositionMessage } from '#lib/domain/defaultLocation.ts';
  import { emptyPlaceValue, type PlaceValue } from '#lib/domain/places.ts';
  import { devicePlace, ipHintFailure, ipPlaceHint, placeSearch } from '#lib/services/locations.ts';
  import { readDefaultLocation, writeDefaultLocation } from '#lib/stores/defaultLocation.svelte.ts';

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
      hint="Used for regional feed and map discovery."
    >
      <LocationPicker
        value={place}
        modes={['physical']}
        onChange={save}
        search={placeSearch.search}
        failure={placeSearch.failure}
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
