<script lang="ts">
  import { untrack } from 'svelte';

  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import type { PhaseStep } from '#lib/domain/projects.ts';

  import '#lib/components/styles/governance.css';

  /**
   * web's return composer (ProjectPhaseChangeSection's revert sheet): pick an earlier planning
   * phase and say why. Members then vote on it; proposing it is not a Yes, as on web.
   */
  interface Props {
    open: boolean;
    phases: PhaseStep[];
    pending: boolean;
    message: string | undefined;
    onSubmit: (to: string, reason: string) => void;
    onClose: () => void;
  }

  let { open, phases, pending, message, onSubmit, onClose }: Props = $props();

  let target = $state('');
  let reason = $state('');
  let local = $state('');

  // Each time it opens, web starts on Production Plan when offered, else the first earlier
  // phase. Later page refreshes (new `phases` arrays) leave the choice alone.
  $effect(() => {
    if (!open) return;
    const offered = untrack(() => phases);
    target = offered.some((p) => p.id === 'phase-2') ? 'phase-2' : (offered[0]?.id ?? '');
    reason = '';
    local = '';
  });

  const targetLabel = $derived(phases.find((p) => p.id === target)?.label ?? target);
  const alert = $derived(local || message);

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    if (!reason.trim()) {
      local = 'Add a reason before submitting this return request.';
      return;
    }
    local = '';
    onSubmit(target, reason.trim());
  }
</script>

<OverlaySheet
  {open}
  title={`Return to ${targetLabel}`}
  labelledById="project-return-phase-sheet"
  rootClass="return-sheet"
  {onClose}
>
  <form id="project-return-phase-form" class="sheet-form" onsubmit={submit}>
    {#if alert}
      <div class="inline-alert" role="alert">{alert}</div>
    {/if}
    <label>
      <span class="field-inline-label">Return to</span>
      <select bind:value={target}>
        {#each phases as phase (phase.id)}
          <option value={phase.id}>{phase.label}</option>
        {/each}
      </select>
    </label>
    <label>
      <span class="field-inline-label">Reason</span>
      <textarea
        bind:value={reason}
        rows="3"
        placeholder="State clearly why the project should return to an earlier planning phase."
      ></textarea>
    </label>
  </form>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={onClose}>Cancel</button>
      <button class="sheet-submit" disabled={pending} form="project-return-phase-form" type="submit"
        >{pending ? 'Working...' : 'Return'}</button
      >
    </div>
  {/snippet}
</OverlaySheet>
