<script lang="ts">
  import PhaseStepper from '#lib/components/governance/PhaseStepper.svelte';

  import type { DetailState } from './detailState.svelte.ts';

  /**
   * web's participation tab, M3a part: the phase stepper and the move to the next phase.
   * Plans, roles and activities join it in M3b and M5.
   */
  let { detail }: { detail: DetailState } = $props();

  let selected = $state('');
  const project = $derived(detail.project);
  const leavingProposal = $derived(project?.phase === 'phase-1');
</script>

{#if project}
  <div class="overview-governance">
    <PhaseStepper
      steps={project.phases}
      current={project.phase}
      selected={selected || project.phase}
      onSelect={(id) => (selected = id)}
    />
    <section class="phase-panel">
      <h2>{project.stage}</h2>
      {#if project.nextPhase && detail.canPropose}
        <p class="phase-note">
          {leavingProposal
            ? 'Planning opens once support is above 66% of all signals and members agree.'
            : `Members vote to move this project to ${project.nextPhase.label}.`}
        </p>
        {#if detail.sheetMessage}<p class="warning-card" role="alert">{detail.sheetMessage}</p>{/if}
        <button
          class="button-primary"
          type="button"
          disabled={detail.saving}
          onclick={() => void detail.proposeNextPhase()}
        >
          {detail.saving ? 'Working...' : `Propose moving to ${project.nextPhase.label}`}
        </button>
      {:else if !project.nextPhase}
        <p class="phase-note">This is the project's last phase.</p>
      {/if}
    </section>
  </div>
{/if}

<style>
  .phase-panel {
    display: grid;
    gap: 10px;
    padding: 14px var(--page-gutter, 16px);
  }
  .phase-panel h2 {
    margin: 0;
    font-size: 18px;
  }
  .phase-note {
    margin: 0;
    color: var(--text-soft);
  }
</style>
