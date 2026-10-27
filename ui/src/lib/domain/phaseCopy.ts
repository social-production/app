import type { PlanKind } from './plans.ts';
import type { CharterMode } from './projects.ts';

/**
 * web-backend's `lifecycle_copy.py` phase summaries (web shows the first sentence under the
 * phase title). Collective and personal services override some phases.
 */
const DEFAULT: Record<string, string> = {
  'phase-1':
    'Define what this project should achieve and how strongly members care about each value.',
  'phase-2': 'Turn agreed values into a concrete production or operations plan.',
  'phase-3': 'Decide how outputs reach the people who need them.',
  'phase-4': 'Prepare inventory, materials, and acquisition steps before execution.',
  'phase-5': 'Schedule and run the activities that deliver the approved plan.',
  'phase-6': 'Confirm execution is complete and ready for final closure.',
  'phase-7': 'The project has finished its lifecycle and is archived for reference.',
};

const OVERRIDES: Partial<Record<CharterMode, Record<string, string>>> = {
  'collective-service': {
    'phase-2': 'Define how the collective service will operate day to day.',
    'phase-3': 'Decide who can access the service and under what conditions.',
  },
  'personal-service': {
    'phase-1': 'One person offers this service.',
    'phase-2': 'This personal service is closed and no longer takes requests.',
  },
  // web-backend `DEFAULT_EVENT_PHASE_COPY` (events use the app's phase ids, ADR 0057).
  event: {
    'phase-1': 'Gather interest and define the values this event should optimize for.',
    'phase-2': 'Propose schedules, locations, and staged plans for the event.',
    'phase-5': 'Run scheduled event activities and fill roles.',
    'phase-7': 'The event has ended; outcomes and history remain visible.',
  },
};

export function phaseSummary(mode: CharterMode | undefined, phaseId: string): string {
  return (mode && OVERRIDES[mode]?.[phaseId]) ?? DEFAULT[phaseId] ?? '';
}

/** web's phaseProgressLabel: the kicker above the phase title. */
export function phaseProgressLabel(phases: { id: string }[], current: string, id: string): string {
  const at = phases.findIndex((p) => p.id === current);
  const i = phases.findIndex((p) => p.id === id);
  if (i === at) return 'Current phase';
  return i < at ? 'Complete' : 'Upcoming';
}

/** sp_governance::phases::signal_gate_met: demand ≥ 66% of all signals, and at least one. */
export function signalGateMet(demand: number, opposition: number): boolean {
  const total = demand + opposition;
  return total > 0 && demand * 10_000 >= 6_600 * total;
}

/** web's projectPlanGateMessage, for the plan kind the next move needs an approved leader of. */
export function planGateMessage(gate: PlanKind, hasPlans: boolean): string {
  // web's EventPhaseChangeSection note.
  if (gate === 'Event') return 'Planning cannot advance until a plan clears quorum and approval.';
  const kind =
    gate === 'Production' || gate === 'Organisation'
      ? 'production or operations plan'
      : 'distribution or access plan';
  return hasPlans
    ? `A ${kind} is waiting on quorum before this project can advance.`
    : `This project needs an approved ${kind} before it can advance.`;
}
