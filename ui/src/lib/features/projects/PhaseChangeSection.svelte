<script lang="ts">
  import PhaseMenu, { type PhaseMenuItem } from '#lib/components/governance/PhaseMenu.svelte';
  import { kindName } from '#lib/domain/governance.ts';
  import { planGateMessage, signalGateMet } from '#lib/domain/phaseCopy.ts';

  import type { DetailState } from './detailState.svelte.ts';

  /**
   * web's ProjectPhaseChangeSection: the gate note and the Phase ▾ menu, on the current phase.
   * web asks for a reason in a sheet before filing; our phase proposals carry none yet, so the
   * menu item files the proposal directly.
   */
  let { detail, shown }: { detail: DetailState; shown: string } = $props();

  const project = $derived(detail.project);
  const currentVisible = $derived(shown === project?.phase);
  const phaseVoteOpen = $derived(
    detail.proposals.some((p) => p.status === 'Open' && kindName(p.kind) === 'phase'),
  );
  const signalGatePasses = $derived(
    project?.phase !== 'phase-1' || signalGateMet(project.demand, project.opposition),
  );
  const gate = $derived(detail.plans?.gate ?? null);
  const planGate = $derived(
    gate && !detail.plans?.gateLeader
      ? planGateMessage(gate, !!detail.plans?.plans.some((p) => p.plan.kind === gate))
      : '',
  );
  const gatePasses = $derived(signalGatePasses && !planGate);
  const closing = $derived(project?.nextPhase?.id === 'phase-7');

  function lockReason(): string {
    if (phaseVoteOpen) return 'A phase vote is already open. Vote on it in Pending votes first.';
    if (!signalGatePasses) return 'Unlocks after this project gets a Support signal.';
    return planGate;
  }

  const items = $derived.by((): PhaseMenuItem[] => {
    const next = project?.nextPhase;
    if (!next || !detail.canPropose) return [];
    const allowed = gatePasses && !phaseVoteOpen && !detail.saving;
    const locked = allowed ? '' : lockReason();
    const onSelect = (): void => void detail.proposeNextPhase();
    return [
      closing
        ? {
            id: 'close',
            glyph: '×',
            label: 'Close project',
            detail: locked || 'Members vote to finish the project. Proposing counts as your Yes.',
            disabled: !allowed,
            onSelect,
          }
        : {
            id: 'advance',
            glyph: '›',
            label: `Advance to ${next.label}`,
            detail: locked || 'Members vote to move on. Proposing counts as your Yes.',
            disabled: !allowed,
            onSelect,
          },
    ];
  });
</script>

{#if project && currentVisible && (phaseVoteOpen || detail.canPropose)}
  <div id="participation-phase-change" class="phase-change-stack">
    {#if project.nextPhase && detail.canPropose && !gatePasses}
      <p class="phase-gate-note">
        {signalGatePasses ? planGate : 'Advance unlocks after this project gets a Support signal.'}
      </p>
    {/if}
    {#if detail.sheetMessage}<p class="warning-card" role="alert">{detail.sheetMessage}</p>{/if}
    {#if items.length > 0}
      <div class="change-action-row">
        <div class="action-group action-group-right">
          <PhaseMenu
            {items}
            currentPhaseLabel={detail.phaseLabel(project.phase)}
            labelledById="project-phase-menu"
          />
        </div>
      </div>
    {/if}
  </div>
{/if}
