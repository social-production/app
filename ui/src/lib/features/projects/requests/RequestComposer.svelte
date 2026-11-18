<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
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

  const showSchedule = $derived(scheduleRequired || !!state.form.scheduledAt || !!state.form.slot);
</script>

<OverlaySheet
  open={state.composing}
  title={t('Request service')}
  labelledById="personal-request-form-sheet"
  onClose={() => (state.composing = false)}
>
  <div class="service-ui">
    <div class="composer-card">
      <div class="request-header-row">
        <div>
          <p>
            {t(
              'Every member of the service sees your request; whoever plans it starts a chat with you.',
            )}
            {#if showSchedule}
              {t('Start from the selected available time and add the details.')}
            {:else}
              {t('Describe what you need so they can review the request and reply.')}
            {/if}
          </p>
        </div>
      </div>
      <input bind:value={state.form.title} maxlength="120" placeholder={t('Request title')} />
      {#if showSchedule}
        <div class="number-grid">
          <label>
            <span class="field-inline-label">{t('Start time')}</span>
            <input
              bind:value={state.form.scheduledAt}
              disabled={!!state.form.slot}
              type="datetime-local"
            />
          </label>
          <label>
            <span class="field-inline-label">{t('Finish time')}</span>
            <input
              bind:value={state.form.endsAt}
              disabled={!!state.form.slot}
              type="datetime-local"
            />
          </label>
        </div>
      {:else}
        <button class="text-button add-time" type="button" onclick={() => state.addPreferredTime()}>
          {t('+ Add a preferred time (optional)')}
        </button>
      {/if}
      <textarea
        bind:value={state.form.body}
        rows="3"
        placeholder={showSchedule
          ? t('What do you need help with? (required)')
          : t('What do you need, and what should they expect? (required)')}></textarea>
      {#if state.error}<p class="validation-copy" role="alert">{state.error}</p>{/if}
    </div>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={() => (state.composing = false)}
        >{t('Cancel')}</button
      >
      <button class="sheet-submit" type="button" disabled={state.busy} onclick={onSend}
        >{state.busy ? t('Sending…') : t('Request')}</button
      >
    </div>
  {/snippet}
</OverlaySheet>
