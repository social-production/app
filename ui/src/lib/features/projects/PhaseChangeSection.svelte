<script lang="ts">
  import { tick } from 'svelte';

  import { portal } from '#lib/components/actions/portal.ts';
  import PhaseMenu, { type PhaseMenuItem } from '#lib/components/governance/PhaseMenu.svelte';
  import ReturnSheet from '#lib/components/governance/ReturnSheet.svelte';
  import { planGateMessage, signalGateMet } from '#lib/domain/phaseCopy.ts';

  import type { DetailState } from './detailState.svelte.ts';

  /**
   * web's ProjectPhaseChangeSection: the gate note, the Phase ▾ menu (advance or close, and
   * return to an earlier phase) and the return sheet, on the current phase. web asks for a reason
   * before advancing too; our advance proposals carry none yet, so that item files directly.
   */
  let { detail, shown }: { detail: DetailState; shown: string } = $props();

  const project = $derived(detail.project);
  const currentVisible = $derived(shown === project?.phase);
  const phaseVoteOpen = $derived(detail.phaseVoteOpen);
  let returning = $state(false);
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

  // web's embedInToolbar: the menu goes into the phase toolbar's end slot when there is one.
  let endHost = $state<HTMLElement | null>(null);
  $effect(() => {
    void shown;
    void tick().then(() => (endHost = document.getElementById('phase-nav-end')));
  });

  function lockReason(): string {
    if (phaseVoteOpen) return 'A phase vote is already open. Vote on it in Pending votes first.';
    if (!signalGatePasses) return 'Unlocks after this project gets a Support signal.';
    return planGate;
  }

  async function fileReturn(to: string, reason: string): Promise<void> {
    if (await detail.proposeReturn(to, reason)) returning = false;
  }

  // web's phaseMenuItems: the move on (advance, or close from the last phase) and the return.
  const items = $derived.by((): PhaseMenuItem[] => {
    if (!detail.canPropose) return [];
    const items: PhaseMenuItem[] = [];
    const next = project?.nextPhase;
    const allowed = gatePasses && !phaseVoteOpen && !detail.saving;
    const locked = allowed ? '' : lockReason();
    const onSelect = (): void => void detail.proposeNextPhase();
    if (next)
      items.push(
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
      );
    if ((project?.returns.length ?? 0) > 0)
      items.push({
        id: 'return',
        glyph: '‹',
        label: 'Return to an earlier phase',
        detail: phaseVoteOpen ? lockReason() : 'Members vote to go back and rework the plans.',
        disabled: phaseVoteOpen || detail.saving,
        onSelect: () => {
          detail.sheetMessage = undefined;
          returning = true;
        },
      });
    return items;
  });
</script>

{#if project && currentVisible && (phaseVoteOpen || detail.canPropose)}
  <div id="participation-phase-change" class="phase-change-stack">
    {#if project.nextPhase && detail.canPropose && !gatePasses}
      <p class="phase-gate-note">
        {signalGatePasses ? planGate : 'Advance unlocks after this project gets a Support signal.'}
      </p>
    {/if}
    {#if detail.sheetMessage && !returning}
      <p class="warning-card" role="alert">{detail.sheetMessage}</p>
    {/if}
    {#if items.length > 0 && endHost}
      <div class="phase-nav-side end" use:portal={endHost}>
        <PhaseMenu
          {items}
          currentPhaseLabel={detail.phaseLabel(project.phase)}
          labelledById="project-phase-menu"
        />
      </div>
    {:else if items.length > 0}
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
  <ReturnSheet
    open={returning}
    phases={project.returns}
    pending={detail.saving}
    message={detail.sheetMessage}
    onSubmit={(to, reason) => void fileReturn(to, reason)}
    onClose={() => (returning = false)}
  />
{/if}
