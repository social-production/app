<script lang="ts">
  import DetailPhaseContext from '#lib/components/governance/DetailPhaseContext.svelte';
  import PhaseStepper from '#lib/components/governance/PhaseStepper.svelte';
  import PhaseWorkToolbar from '#lib/components/governance/PhaseWorkToolbar.svelte';
  import { phaseProgressLabel, phaseSummary } from '#lib/domain/phaseCopy.ts';
  import { planKindFor } from '#lib/domain/plans.ts';

  import ActivityPhase from './ActivityPhase.svelte';
  import type { DetailState } from './detailState.svelte.ts';
  import PhaseChangeSection from './PhaseChangeSection.svelte';
  import PlanPhase from './PlanPhase.svelte';
  import ValuePhase from './ValuePhase.svelte';

  /**
   * web's ProjectLifecyclePanel: the phase stepper, the selected phase's context (kicker, title,
   * summary), then its work (values in Proposal, plans, M3b; activities and roles in the
   * Activity phase, M5b) and the phase-change menu.
   */
  let { detail }: { detail: DetailState } = $props();

  let selected = $state('');
  const project = $derived(detail.project);
  const shown = $derived(selected || project?.phase || 'phase-1');
  const planKind = $derived(project?.charter ? planKindFor(project.charter.mode, shown) : null);
  // web's activity phase: phase-5, or phase-1 for a personal service.
  const personal = $derived(project?.charter?.mode === 'personal-service');
  const activityPhase = $derived(personal ? shown === 'phase-1' : shown === 'phase-5');
</script>

{#if project}
  <section class="lifecycle-shell overview-governance">
    <PhaseStepper
      steps={project.phases}
      current={project.phase}
      selected={shown}
      onSelect={(id) => (selected = id)}
    />
    <DetailPhaseContext
      kicker={phaseProgressLabel(project.phases, project.phase, shown)}
      title={detail.phaseLabel(shown)}
      summary={phaseSummary(project.charter?.mode, shown)}
    />
    <section class="phase-panel overview-phase-work">
      {#if planKind}
        <PlanPhase {detail} kind={planKind} />
      {:else if activityPhase}
        <ActivityPhase {detail} current={project.phase === shown} />
      {:else if shown === 'phase-1' && project.charter?.mode !== 'personal-service'}
        <ValuePhase {detail} />
      {:else}
        <!-- Each of web's phase contents carries a toolbar; it gives the Phase menu its place
             in the dock. -->
        <PhaseWorkToolbar />
      {/if}
      <PhaseChangeSection {detail} {shown} />
    </section>
  </section>
{/if}
