<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { WEEKDAYS } from '#lib/domain/availability.ts';

  import type { AvailabilityState } from './availabilityState.svelte.ts';

  /**
   * web's personal-availability-sheet ("Add availability", ADR 0077): a one-off slot or weekly
   * hours. Times are the browser's until a Display timezone setting exists (PARITY), so web's
   * Timezone selects are left out; weekly hours keep the browser's zone.
   */
  interface Props {
    state: AvailabilityState;
    onOneOff: () => void;
    onWeekly: () => void;
  }

  let { state, onOneOff, onWeekly }: Props = $props();
</script>

<OverlaySheet
  open={state.open}
  title="Add availability"
  labelledById="personal-availability-sheet"
  onClose={() => (state.open = false)}
>
  <div class="service-ui">
    <div class="composer-card">
      <div class="request-header-row">
        <div>
          <p>One-off slots and weekly hours are what other people can request.</p>
        </div>
      </div>
      <div class="binary-row">
        <button
          class="vote-chip"
          class:selected={state.kind === 'one-off'}
          type="button"
          onclick={() => (state.kind = 'one-off')}>One-off</button
        >
        <button
          class="vote-chip"
          class:selected={state.kind === 'weekly'}
          type="button"
          onclick={() => (state.kind = 'weekly')}>Weekly</button
        >
      </div>
      {#if state.kind === 'one-off'}
        <div class="number-grid">
          <label>
            <span class="field-inline-label">Start time</span>
            <input bind:value={state.oneOff.scheduledAt} type="datetime-local" />
          </label>
          <label>
            <span class="field-inline-label">Finish time</span>
            <input bind:value={state.oneOff.endsAt} type="datetime-local" />
          </label>
        </div>
      {:else}
        <label>
          <span class="field-inline-label">Weekday</span>
          <select bind:value={state.weekly.weekday}>
            {#each WEEKDAYS as label, value (value)}
              <option {value}>{label}</option>
            {/each}
          </select>
        </label>
        <div class="number-grid">
          <label>
            <span class="field-inline-label">Start</span>
            <input bind:value={state.weekly.start} type="time" />
          </label>
          <label>
            <span class="field-inline-label">End</span>
            <input bind:value={state.weekly.end} type="time" />
          </label>
        </div>
        <input bind:value={state.weekly.note} maxlength="120" placeholder="Optional note" />
      {/if}
      {#if state.error}<p class="validation-copy" role="alert">{state.error}</p>{/if}
    </div>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={() => (state.open = false)}>Cancel</button
      >
      {#if state.kind === 'one-off'}
        <button class="sheet-submit" type="button" disabled={state.busy} onclick={onOneOff}
          >Add availability</button
        >
      {:else}
        <button class="sheet-submit" type="button" disabled={state.busy} onclick={onWeekly}
          >Add weekly hours</button
        >
      {/if}
    </div>
  {/snippet}
</OverlaySheet>
