<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { VALUE_LABEL_MAX } from '#lib/domain/values.ts';

  import '#lib/components/styles/values.css';

  /** web's AddValueSheet: one line, "What should this achieve?". */
  interface Props {
    open: boolean;
    pending: boolean;
    message: string | undefined;
    onSubmit: (label: string) => void;
    onClose: () => void;
  }

  let { open, pending, message, onSubmit, onClose }: Props = $props();

  let value = $state('');

  $effect(() => {
    if (open) value = '';
  });

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    onSubmit(value.trim());
  }
</script>

<OverlaySheet
  {open}
  title="Add value"
  labelledById="add-value-sheet"
  rootClass="value-sheet"
  {onClose}
>
  <form id="add-value-form" class="add-value-form" onsubmit={submit}>
    {#if message}<div class="warning-card" role="alert">{message}</div>{/if}
    <label class="field">
      <span class="field-label">What should this achieve?</span>
      <input
        bind:value
        maxlength={VALUE_LABEL_MAX}
        placeholder="Add a value, for example: should make use of unused space"
      />
    </label>
  </form>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={onClose}>Cancel</button>
      <button
        class="sheet-submit"
        disabled={pending || !value.trim()}
        form="add-value-form"
        type="submit">{pending ? 'Working...' : 'Add value'}</button
      >
    </div>
  {/snippet}
</OverlaySheet>
