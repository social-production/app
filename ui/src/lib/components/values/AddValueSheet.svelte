<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { IMPORTANCE_OPTIONS, VALUE_LABEL_MAX, type Importance } from '#lib/domain/values.ts';

  import DiscreteScale from './DiscreteScale.svelte';

  import '#lib/components/styles/values.css';

  /** web's AddValueSheet, with your own rating in the same step so a new value never sits
   * unrated by its author (Plan, Phase and Feedback Screens, 2). */
  interface Props {
    open: boolean;
    pending: boolean;
    message: string | undefined;
    onSubmit: (label: string, importance: Importance) => void;
    onClose: () => void;
  }

  let { open, pending, message, onSubmit, onClose }: Props = $props();

  let value = $state('');
  let importance = $state<Importance | 0>(0);

  $effect(() => {
    if (!open) return;
    value = '';
    importance = 0;
  });

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    if (importance) onSubmit(value.trim(), importance);
  }
</script>

<OverlaySheet
  {open}
  title={t('Add value')}
  labelledById="add-value-sheet"
  rootClass="value-sheet"
  {onClose}
>
  <form id="add-value-form" class="add-value-form" onsubmit={submit}>
    {#if message}<div class="warning-card" role="alert">{message}</div>{/if}
    <label class="field">
      <span class="field-label">{t('What should this achieve?')}</span>
      <input
        bind:value
        maxlength={VALUE_LABEL_MAX}
        placeholder={t('Add a value, for example: should make use of unused space')}
      />
    </label>
    <div class="field">
      <span class="field-label">{t('How much does it matter to you?')}</span>
      <DiscreteScale
        options={IMPORTANCE_OPTIONS}
        selectedValue={importance}
        leftLabel={t('Unnecessary')}
        rightLabel={t('Required')}
        onSelect={(n) => (importance = n as Importance)}
      />
    </div>
  </form>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={onClose}>{t('Cancel')}</button>
      <button
        class="sheet-submit"
        disabled={pending || !value.trim() || !importance}
        form="add-value-form"
        type="submit">{pending ? t('Working...') : t('Add value')}</button
      >
    </div>
  {/snippet}
</OverlaySheet>
