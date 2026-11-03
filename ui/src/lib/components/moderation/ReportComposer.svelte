<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import type { ReportReason } from '#lib/domain/moderation.ts';

  import '#lib/components/styles/report.css';

  /** web's ReportComposerModal: the reason and what's wrong, then Submit report. */
  interface Props {
    open: boolean;
    itemLabel: string;
    pending: boolean;
    /** Why the last submit was refused. */
    error: string;
    reason: ReportReason;
    description: string;
    onClose: () => void;
    onSubmit: () => void;
  }

  let {
    open,
    itemLabel,
    pending,
    error,
    reason = $bindable(),
    description = $bindable(),
    onClose,
    onSubmit,
  }: Props = $props();
</script>

<OverlaySheet
  {open}
  title={`Report ${itemLabel}`}
  labelledById="report-composer-title"
  rootClass="report-composer"
  elevated
  {onClose}
>
  <div class="report-body">
    {#if error}<div class="warning-card" role="alert">{error}</div>{/if}
    <label class="field-stack">
      <span class="field-label">Reason</span>
      <select bind:value={reason}>
        <option value="spam">Spam</option>
        <option value="serious-harm">Serious harm</option>
      </select>
    </label>
    <label class="field-stack">
      <span class="field-label">What's wrong?</span>
      <textarea bind:value={description} placeholder="Add context for the report..." rows="4"
      ></textarea>
    </label>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button
        class="sheet-submit"
        disabled={pending || !description.trim()}
        type="button"
        onclick={onSubmit}
      >
        {pending ? 'Sending…' : 'Submit report'}
      </button>
    </div>
  {/snippet}
</OverlaySheet>
