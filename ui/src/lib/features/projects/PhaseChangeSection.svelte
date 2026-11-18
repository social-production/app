<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { tick } from 'svelte';

  import { portal } from '#lib/components/actions/portal.ts';
  import PhaseMenu, { type PhaseMenuItem } from '#lib/components/governance/PhaseMenu.svelte';
  import PhaseMoveSheet, { type PhaseMove } from '#lib/components/governance/PhaseMoveSheet.svelte';
  import { planGateMessage, signalGateMet } from '#lib/domain/phaseCopy.ts';

  import type { DetailState } from './detailState.svelte.ts';

  /**
   * web's ProjectPhaseChangeSection: the gate note and the Phase ▾ button, which opens every
   * move (advance or close, and each return) on one sheet with the reason (Plan, Phase and
   * Feedback Screens, 4).
   */
  let { detail, shown }: { detail: DetailState; shown: string } = $props();

  const project = $derived(detail.project);
  const currentVisible = $derived(shown === project?.phase);
  const phaseVoteOpen = $derived(detail.phaseVoteOpen);
  let moving = $state(false);
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
  // web's EventPhaseChangeSection words the menu for events (ADR 0057, 0061).
  const event = $derived(project?.charter?.mode === 'event');
  const how = $derived(
    project?.governance === 'organizer_controlled'
      ? 'Organizers decide; members are notified.'
      : 'Members vote. Proposing counts as your Yes.',
  );

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

  async function file(move: PhaseMove, reason: string): Promise<void> {
    const done = move.convert
      ? await detail.proposeConvert(reason)
      : move.to
        ? await detail.proposeReturn(move.to, reason)
        : await detail.proposeNextPhase(reason);
    if (done) moving = false;
  }

  // Every move on one sheet: the move on (advance, or close from the last phase), then each
  // earlier phase it can return to.
  const moves = $derived.by((): PhaseMove[] => {
    if (!detail.canPropose || !project) return [];
    const moves: PhaseMove[] = [];
    const next = project.nextPhase;
    const allowed = gatePasses && !phaseVoteOpen && !detail.saving;
    const locked = allowed ? '' : lockReason();
    if (next)
      moves.push({
        id: 'advance',
        label: closing ? (event ? 'Close event' : 'Close project') : `Advance to ${next.label}`,
        detail:
          locked ||
          (closing
            ? event
              ? `Ends the event and keeps its record. ${how}`
              : 'Ends the work here. Members vote on it.'
            : event
              ? how
              : 'Members vote to move on. Proposing counts as your Yes.'),
        disabled: !allowed,
        danger: closing,
      });
    // Convert (ADR 0106): any phase after Proposal, until it closes; not for events.
    if (!event && project.phase !== 'phase-1' && project.phase !== 'phase-7')
      moves.push({
        id: 'convert',
        label: 'Convert to a new proposal',
        detail: phaseVoteOpen
          ? lockReason()
          : 'Closes this and continues its work as a new proposal with the same title, place and tags. Members vote on it.',
        disabled: phaseVoteOpen || detail.saving,
        danger: true,
        convert: true,
      });
    for (const phase of project.returns)
      moves.push({
        id: `return-${phase.id}`,
        label: `Return to ${phase.label}`,
        detail: phaseVoteOpen
          ? lockReason()
          : event
            ? `Go back and rework the plan. ${how}`
            : 'Members vote to go back and rework the plans.',
        disabled: phaseVoteOpen || detail.saving,
        to: phase.id,
      });
    return moves;
  });

  const open = (): void => {
    detail.sheetMessage = undefined;
    moving = true;
  };
  const items = $derived<PhaseMenuItem[]>(
    moves.length ? [{ id: 'moves', glyph: '›', label: 'Phase', detail: '', onSelect: open }] : [],
  );
</script>

{#if project && currentVisible && (phaseVoteOpen || detail.canPropose)}
  <div id="participation-phase-change" class="phase-change-stack">
    {#if project.nextPhase && detail.canPropose && !gatePasses}
      <p class="phase-gate-note">
        {signalGatePasses
          ? planGate
          : t('Advance unlocks after this project gets a Support signal.')}
      </p>
    {/if}
    {#if detail.sheetMessage && !moving}
      <p class="warning-card" role="alert">{detail.sheetMessage}</p>
    {/if}
    {#if items.length > 0 && endHost}
      <div class="phase-nav-side end" use:portal={endHost}>
        <PhaseMenu {items} onOpen={open} />
      </div>
    {:else if items.length > 0}
      <div class="change-action-row">
        <div class="action-group action-group-right">
          <PhaseMenu {items} onOpen={open} />
        </div>
      </div>
    {/if}
  </div>
  <PhaseMoveSheet
    open={moving}
    {moves}
    pending={detail.saving}
    message={detail.sheetMessage}
    onSubmit={(move, reason) => void file(move, reason)}
    onClose={() => (moving = false)}
  />
{/if}
