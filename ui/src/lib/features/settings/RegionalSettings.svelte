<script lang="ts">
  import LocationPicker from '#lib/components/location/LocationPicker.svelte';
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

  function save(next: PlaceValue): void {
    message = '';
    place = next;
    // web clears the default when the field is emptied.
    writeDefaultLocation(me, next.displayLabel.trim() ? next : emptyPlaceValue(), device);
  }

  async function fromNetwork(): Promise<void> {
    message = '';
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
    <div class="setting-item">
      <div>
        <strong>Default location</strong>
        <p>Used for regional feed and map discovery.</p>
      </div>
      <LocationPicker
        value={place}
        modes={['physical']}
        onChange={save}
        search={placeSearch.search}
        failure={placeSearch.failure}
      />
    </div>
    <div class="setting-item">
      <div>
        <strong>Device location (GPS)</strong>
        <p>
          Permission for the map’s “Use my location” button. Uses your device GPS when you tap it —
          not applied automatically in the background.
        </p>
      </div>
      <button
        aria-checked={device}
        aria-label="Device location (GPS)"
        class="switch"
        class:on={device}
        role="switch"
        type="button"
        onclick={() => void toggleDevice()}
      >
        <span class="switch-thumb"></span>
        <span class="sr-only">{device ? 'On' : 'Off'}</span>
      </button>
    </div>
    <div class="setting-item">
      <div>
        <strong>Approximate network location</strong>
        <p>
          One-time set of your default place from your public IP when GPS isn’t available or you
          don’t want it. Different from GPS — coarser, and only runs when you press the button.
        </p>
      </div>
      <button class="button-secondary" type="button" onclick={() => void fromNetwork()}>
        Set from network
      </button>
    </div>
    {#if message}<p class="status error" role="alert">{message}</p>{/if}
  </div>
</section>
