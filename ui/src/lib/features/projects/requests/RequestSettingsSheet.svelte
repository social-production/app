<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import type { RequestSettings } from '#lib/domain/projects.ts';

  import type { ServiceRequestsState } from './serviceRequestsState.svelte.ts';

  /** web's personal-request-settings-sheet: the creator's settings apply at once. */
  interface Props {
    state: ServiceRequestsState;
    current: RequestSettings;
    onSave: () => void;
  }

  let { state, current, onSave }: Props = $props();

  const form = $derived(state.settingsForm);
  // web's `requestSettingsCanSubmit`: something has to change.
  const changed = $derived(
    form.enabled !== current.enabled ||
      form.mode !== current.mode ||
      form.offSchedule !== current.offSchedule,
  );
</script>

<OverlaySheet
  open={state.settingsOpen}
  title="Request settings"
  labelledById="personal-request-settings-sheet"
  onClose={() => (state.settingsOpen = false)}
>
  <div class="service-ui">
    <div class="composer-card">
      <div class="request-header-row">
        <div>
          <p>
            These settings apply immediately because the project creator is the only person running
            this service.
          </p>
        </div>
      </div>

      <label class="checkbox-row">
        <input bind:checked={state.settingsForm.enabled} type="checkbox" />
        <span>Enable requests</span>
      </label>

      {#if form.enabled}
        <label>
          <span class="field-inline-label">Request mode</span>
          <select bind:value={state.settingsForm.mode}>
            <option value="calendar">Scheduled slots only</option>
            <option value="direct">Message requests only</option>
            <option value="both">Scheduled slots and message requests</option>
          </select>
        </label>
        {#if form.mode === 'calendar' || form.mode === 'both'}
          <label class="checkbox-row">
            <input bind:checked={state.settingsForm.offSchedule} type="checkbox" />
            <span>Allow off-schedule requests</span>
          </label>
        {/if}
        <p class="field-help">
          Scheduled slots start from listed times. Message requests let people write in without
          choosing a listed slot.
          {#if form.mode !== 'direct'}
            Off-schedule requests can ask for a time that is not already listed.
          {/if}
        </p>
      {/if}
      {#if state.error}<p class="validation-copy" role="alert">{state.error}</p>{/if}
    </div>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={() => (state.settingsOpen = false)}
        >Cancel</button
      >
      <button class="sheet-submit" disabled={!changed || state.busy} type="button" onclick={onSave}
        >Save settings</button
      >
    </div>
  {/snippet}
</OverlaySheet>
