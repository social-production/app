<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import type { RequestSettings } from '#lib/domain/projects.ts';

  import type { ServiceRequestsState } from './serviceRequestsState.svelte.ts';

  /** web's personal-request-settings-sheet: the creator's settings apply at once. With
   * `vote`, web's collective-request-settings-sheet: a change members vote on, with a reason. */
  interface Props {
    state: ServiceRequestsState;
    current: RequestSettings;
    onSave: () => void;
    vote?: boolean;
  }

  let { state, current, onSave, vote = false }: Props = $props();

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
  title={t('Request settings')}
  labelledById="personal-request-settings-sheet"
  onClose={() => (state.settingsOpen = false)}
>
  <div class="service-ui">
    <div class="composer-card">
      <div class="request-header-row">
        <div>
          <p>
            {#if vote}
              {t(
                'Each vote runs on its own and applies automatically once it reaches 66% approval and the required vote count.',
              )}
            {:else}
              {t(
                'These settings apply immediately because the project creator is the only person running this service.',
              )}
            {/if}
          </p>
        </div>
      </div>

      <label class="checkbox-row">
        <input bind:checked={state.settingsForm.enabled} type="checkbox" />
        <span>{t('Enable requests')}</span>
      </label>

      {#if form.enabled}
        <label>
          <span class="field-inline-label">{t('Request mode')}</span>
          <select bind:value={state.settingsForm.mode}>
            <option value="calendar">{t('Scheduled slots only')}</option>
            <option value="direct">{t('Message requests only')}</option>
            <option value="both">{t('Scheduled slots and message requests')}</option>
          </select>
        </label>
        {#if vote ? form.mode === 'both' : form.mode !== 'direct'}
          <label class="checkbox-row">
            <input bind:checked={state.settingsForm.offSchedule} type="checkbox" />
            <span
              >{vote
                ? t('Allow message requests when no slot is listed')
                : t('Allow off-schedule requests')}</span
            >
          </label>
        {/if}
        <p class="field-help">
          {t(
            'Scheduled slots start from listed times. Message requests let people write in without choosing a listed slot.',
          )}
          {#if form.mode !== 'direct' && !vote}
            {t('Off-schedule requests can ask for a time that is not already listed.')}
          {/if}
        </p>
      {/if}
      {#if vote}
        {#if !changed}
          <p class="field-help">
            {t('Choose a different request setup before adding a reason or starting a vote.')}
          </p>
        {/if}
        <label>
          <span class="field-inline-label">{t('Reason')}</span>
          <textarea
            bind:value={state.settingsReason}
            disabled={!changed}
            rows="3"
            placeholder={changed
              ? t('Explain why the request flow should change right now.')
              : t('Choose a different request setup to unlock the reason field.')}></textarea>
        </label>
      {/if}
      {#if state.error}<p class="validation-copy" role="alert">{state.error}</p>{/if}
    </div>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={() => (state.settingsOpen = false)}
        >{t('Cancel')}</button
      >
      <button
        class="sheet-submit"
        disabled={!changed || state.busy || (vote && !state.settingsReason.trim())}
        type="button"
        onclick={onSave}>{vote ? t('Start vote') : t('Save settings')}</button
      >
    </div>
  {/snippet}
</OverlaySheet>
