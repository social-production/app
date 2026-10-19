<script lang="ts">
  import DetailPhaseContext from '#lib/components/governance/DetailPhaseContext.svelte';
  import PhaseStepper from '#lib/components/governance/PhaseStepper.svelte';
  import { phaseProgressLabel, phaseSummary } from '#lib/domain/phaseCopy.ts';
  import { planKindFor } from '#lib/domain/plans.ts';

  import type { DetailState } from './detailState.svelte.ts';
  import PhaseChangeSection from './PhaseChangeSection.svelte';
  import PlanPhase from './PlanPhase.svelte';

  /**
   * web's ProjectLifecyclePanel: the phase stepper, the selected phase's context (kicker, title,
   * summary), then its work (plans, M3b) and the phase-change menu. Roles and activities join
   * the work in M5.
   */
  let { detail }: { detail: DetailState } = $props();

  let selected = $state('');
  const project = $derived(detail.project);
  const shown = $derived(selected || project?.phase || 'phase-1');
  const planKind = $derived(project?.charter ? planKindFor(project.charter.mode, shown) : null);
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
      {/if}
      <PhaseChangeSection {detail} {shown} />
    </section>
  </section>
{/if}
