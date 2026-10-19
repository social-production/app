<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  import '#lib/components/styles/governance.css';

  /**
   * web's next-phase composer (ProjectPhaseChangeSection's advance sheet): say why the project
   * should move on (or close). Proposing counts as the proposer's Yes, as on web. Its styles are
   * the same ported block as the return sheet's (`.return-sheet`).
   */
  interface Props {
    open: boolean;
    /** "Advance to Production Plan" or "Close project". */
    title: string;
    closing: boolean;
    pending: boolean;
    message: string | undefined;
    onSubmit: (reason: string) => void;
    onClose: () => void;
  }

  let { open, title, closing, pending, message, onSubmit, onClose }: Props = $props();

  let reason = $state('');
  let local = $state('');

  $effect(() => {
    if (!open) return;
    reason = '';
    local = '';
  });

  const alert = $derived(local || message);

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    if (!reason.trim()) {
      local = 'Add a reason before submitting this phase change.';
      return;
    }
    local = '';
    onSubmit(reason.trim());
  }
</script>

<OverlaySheet
  {open}
  {title}
  labelledById="project-advance-phase-sheet"
  rootClass="return-sheet"
  {onClose}
>
  <form id="project-advance-phase-form" class="sheet-form" onsubmit={submit}>
    {#if alert}
      <div class="inline-alert" role="alert">{alert}</div>
    {/if}
    <label>
      <span class="field-inline-label">Reason</span>
      <textarea
        bind:value={reason}
        rows="3"
        placeholder={closing
          ? 'State why the project should close or where the work continues next.'
          : 'State why this phase change should happen now.'}></textarea>
    </label>
  </form>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={onClose}>Cancel</button>
      <button
        class="sheet-submit"
        disabled={pending}
        form="project-advance-phase-form"
        type="submit">{pending ? 'Working...' : title}</button
      >
    </div>
  {/snippet}
</OverlaySheet>
