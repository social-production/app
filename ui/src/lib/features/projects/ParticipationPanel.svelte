<script lang="ts">
  import ChallengeList from '#lib/components/governance/ChallengeList.svelte';
  import PhaseStepper from '#lib/components/governance/PhaseStepper.svelte';
  import ReturnSheet from '#lib/components/governance/ReturnSheet.svelte';
  import { PLAN_KIND_NAMES, planKindFor } from '#lib/domain/plans.ts';

  import type { DetailState } from './detailState.svelte.ts';
  import PlanPhase from './PlanPhase.svelte';

  /**
   * web's participation tab: the phase stepper, the selected phase's plans (M3b), the move to
   * the next phase and the return to an earlier one (M3c), and counts members can challenge.
   * Roles and activities join it in M5.
   */
  let { detail }: { detail: DetailState } = $props();

  let selected = $state('');
  let returning = $state(false);
  let challenging = $state(false);
  const project = $derived(detail.project);
  const shown = $derived(selected || project?.phase || 'phase-1');
  const leavingProposal = $derived(project?.phase === 'phase-1');
  const planKind = $derived(project?.charter ? planKindFor(project.charter.mode, shown) : null);
  const gate = $derived(detail.plans?.gate ?? null);
  // web's `_ensure_project_phase_plan_gate`: leaving a plan phase needs an approved leader.
  const gated = $derived(!!gate && !detail.plans?.gateLeader);
  const canReturn = $derived(
    shown === project?.phase && detail.canPropose && (project?.returns.length ?? 0) > 0,
  );

  async function fileReturn(to: string, reason: string): Promise<void> {
    if (await detail.proposeReturn(to, reason)) returning = false;
  }

  async function challenge(id: string): Promise<void> {
    challenging = true;
    await detail.challenge(id);
    challenging = false;
  }
</script>

{#if project}
  <div class="overview-governance">
    <PhaseStepper
      steps={project.phases}
      current={project.phase}
      selected={shown}
      onSelect={(id) => (selected = id)}
    />
    <section class="phase-panel">
      <h2>{project.phases.find((p) => p.id === shown)?.label ?? project.stage}</h2>
      {#if planKind}
        <PlanPhase {detail} kind={planKind} />
      {/if}
      <!-- Moves happen from the current phase only. -->
      {#if shown === project.phase && project.nextPhase && detail.canPropose}
        <p class="phase-note">
          {#if leavingProposal}
            Planning opens once support is above 66% of all signals and members agree.
          {:else if gated && gate}
            An approved {PLAN_KIND_NAMES[gate]} is required before advancing this project phase.
          {:else}
            Members vote to move this project to {project.nextPhase.label}.
          {/if}
        </p>
        {#if detail.sheetMessage && !returning}<p class="warning-card" role="alert">
            {detail.sheetMessage}
          </p>{/if}
        <button
          class="button-primary"
          type="button"
          disabled={detail.saving || gated}
          onclick={() => void detail.proposeNextPhase()}
        >
          {detail.saving ? 'Working...' : `Propose moving to ${project.nextPhase.label}`}
        </button>
      {:else if shown === project.phase && !project.nextPhase}
        <p class="phase-note">This is the project's last phase.</p>
      {/if}
      {#if canReturn}
        <!-- web's PhaseMenu "Return to an earlier phase" item and its lock reason. -->
        <p class="phase-note">
          {detail.phaseVoteOpen
            ? 'A phase vote is already open. Vote on it in Pending votes first.'
            : 'Members vote to go back and rework the plans.'}
        </p>
        <button
          class="button-secondary"
          type="button"
          disabled={detail.saving || detail.phaseVoteOpen}
          onclick={() => {
            detail.sheetMessage = undefined;
            returning = true;
          }}>Return to an earlier phase</button
        >
      {/if}
      {#if detail.project?.amMember}
        <ChallengeList
          proposals={detail.challengeable}
          phaseLabel={detail.phaseLabel}
          pending={challenging}
          onChallenge={(p) => void challenge(p.id)}
        />
      {/if}
    </section>
  </div>
  <ReturnSheet
    open={returning}
    phases={project.returns}
    pending={detail.saving}
    message={detail.sheetMessage}
    onSubmit={(to, reason) => void fileReturn(to, reason)}
    onClose={() => (returning = false)}
  />
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
