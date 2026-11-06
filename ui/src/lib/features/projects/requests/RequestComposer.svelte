<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  import type { ServiceRequestsState } from './serviceRequestsState.svelte.ts';

  /**
   * web's personal-request-form-sheet ("Request service"). Times are the browser's until a
   * Display timezone setting exists (PARITY), so web's Timezone select is left out.
   */
  interface Props {
    state: ServiceRequestsState;
    /** web's `requiresSchedule`: a calendar-only service always asks for a time. */
    scheduleRequired: boolean;
    onSend: () => void;
  }

  let { state, scheduleRequired, onSend }: Props = $props();

  const showSchedule = $derived(scheduleRequired || !!state.form.scheduledAt);
</script>

<OverlaySheet
  open={state.composing}
  title="Request service"
  labelledById="personal-request-form-sheet"
  onClose={() => (state.composing = false)}
>
  <div class="service-ui">
    <div class="composer-card">
      <div class="request-header-row">
        <div>
          <p>
            Sending a request opens a one-to-one chat with the creator in Messages.
            {#if showSchedule}
              Start from the selected available time and add the details for the creator.
            {:else}
              Describe what you need so the creator can review the request and reply.
            {/if}
          </p>
        </div>
      </div>
      <input bind:value={state.form.title} maxlength="120" placeholder="Request title" />
      {#if showSchedule}
        <div class="number-grid">
          <label>
            <span class="field-inline-label">Start time</span>
            <input bind:value={state.form.scheduledAt} type="datetime-local" />
          </label>
          <label>
            <span class="field-inline-label">Finish time</span>
            <input bind:value={state.form.endsAt} type="datetime-local" />
          </label>
        </div>
      {:else}
        <button class="text-button add-time" type="button" onclick={() => state.addPreferredTime()}>
          + Add a preferred time (optional)
        </button>
      {/if}
      <textarea
        bind:value={state.form.body}
        rows="3"
        placeholder={showSchedule
          ? 'What do you need help with? (required)'
          : 'What do you need, and what should the creator expect? (required)'}></textarea>
      {#if state.error}<p class="validation-copy" role="alert">{state.error}</p>{/if}
    </div>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={() => (state.composing = false)}
        >Cancel</button
      >
      <button class="sheet-submit" type="button" disabled={state.busy} onclick={onSend}
        >{state.busy ? 'Sending…' : 'Request'}</button
      >
    </div>
  {/snippet}
</OverlaySheet>
