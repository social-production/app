/**
 * Project plans (M3b, ADR 0039). Mirrors the governance zome's plan board; wording and wizard
 * steps are web's (web/src/lib/utils/planRubric.ts, ProjectLifecyclePlanPhaseContent.svelte).
 */

import type { EventSchedule } from './eventSchedule.ts';
import type { Place } from './places.ts';
import type { ProposalStatus, VoteChoice } from './governance.ts';
import type { CharterMode, ProjectMode, ProjectSubtype } from './projects.ts';

/** sp_governance::plans::PlanKind ("Organisation" is web's operations plan). */
export type PlanKind = 'Production' | 'Distribution' | 'Organisation' | 'Access' | 'Event';
export type PlanRating = 1 | 2 | 3 | 4 | 5;

export interface PlanStage {
  title: string;
  details: string;
  start_date: string | null;
  end_date: string | null;
}

/** project_integrity::Plan, as sent and stored. */
export interface Plan {
  kind: PlanKind;
  title: string;
  description: string;
  demand_note: string;
  subtype: ProjectSubtype | null;
  repository_url: string | null;
  location: string | null;
  /** The place behind `location` (ADR 0078). */
  place?: Place | null;
  recipients: string | null;
  stages: PlanStage[];
  /** Event plans only (ADR 0057). */
  schedule?: EventSchedule | null;
}

export interface PlanCriterion {
  id: string;
  label: string;
  /** A rubric question, or one of the project's prominent values (ADR 0041). */
  kind: 'Rubric' | 'Value';
  mine: PlanRating | null;
  count: number;
  /** 0 when unrated. */
  average: number;
}

export interface PlanTally {
  yes: number;
  no: number;
  neutral: number;
  population: number;
  required: number;
  status: ProposalStatus;
  myVote: VoteChoice | null;
  settled: boolean;
}

export interface PlanView {
  id: string;
  author: string;
  createdAt: number;
  plan: Plan;
  tally: PlanTally;
  /** Passed, and still passes counting only Full members' votes. */
  approved: boolean;
  leader: 'Leading' | 'Tied' | null;
  criteria: PlanCriterion[];
  average: number | null;
}

export interface PlanBoard {
  plans: PlanView[];
  /** The kind members can submit now (the current phase's). */
  openKind: PlanKind | null;
  /** Every kind this phase takes: one, or a proposal's three type-giving kinds (ADR 0059). */
  openKinds: PlanKind[];
  /** The kind whose approved leader the next phase move needs, and that leader. */
  gate: PlanKind | null;
  gateLeader: string | null;
  members: number;
  required: number;
}

/** "production plan", "operations plan"… (web's submit labels). */
export const PLAN_KIND_NAMES: Record<PlanKind, string> = {
  Production: 'production plan',
  Distribution: 'distribution plan',
  Organisation: 'operations plan',
  Access: 'access plan',
  Event: 'event plan',
};

/** web-backend `_subtype_label`. */
export const SUBTYPE_LABELS: Record<ProjectSubtype, string> = {
  standard: 'Standard',
  software: 'Software',
  'asset-management': 'Asset management',
};

/** web's `projectSubtypeOptions`, for the wizard's subtype step. */
export function subtypeOptions(mode: ProjectMode): { value: ProjectSubtype; label: string }[] {
  const noun = mode === 'collective-service' ? 'service' : 'project';
  return [
    { value: 'standard', label: `Standard ${noun}` },
    { value: 'software', label: `Software ${noun}` },
  ];
}

/** What a proposal becomes when a plan of this kind leads it out of its Plan phase
 * (sp_governance `PlanKind::makes`, ADR 0059), with the wizard's choice copy. */
export const PLAN_MAKES: {
  value: PlanKind;
  mode: ProjectMode | 'event';
  label: string;
  summary: string;
  makes: string;
}[] = [
  {
    value: 'Production',
    mode: 'productive',
    label: 'Production plan',
    summary: 'Build or produce something new. Makes this a productive project.',
    makes: 'a productive project',
  },
  {
    value: 'Organisation',
    mode: 'collective-service',
    label: 'Operations plan',
    summary: 'Run or maintain something in use. Makes this a collective service.',
    makes: 'a collective service',
  },
  {
    value: 'Event',
    mode: 'event',
    label: 'Event plan',
    summary: 'A dated occasion with a place. Makes this a public event.',
    makes: 'a public event',
  },
];

/** "Production plan · makes this a productive project", for a plan on a proposal. */
export function planMakesLine(kind: PlanKind): string | null {
  const made = PLAN_MAKES.find((m) => m.value === kind);
  return made ? `${made.label} · makes this ${made.makes}` : null;
}

/** The plan kind a phase of this mode takes (sp_governance `PlanKind::for_phase`). A
 * proposal's Plan phase takes three; this names the first (see `PlanBoard.openKinds`). */
export function planKindFor(mode: CharterMode, phase: string): PlanKind | null {
  if (mode === 'proposal') return phase === 'phase-2' ? 'Production' : null;
  if (mode === 'event') return phase === 'phase-2' ? 'Event' : null;
  const collective = mode === 'collective-service';
  if (mode === 'personal-service') return null;
  if (phase === 'phase-2') return collective ? 'Organisation' : 'Production';
  if (phase === 'phase-3') return collective ? 'Access' : 'Distribution';
  return null;
}

/** The phase a plan kind belongs to (the reverse of `planKindFor`). */
export const planPhaseOf = (kind: PlanKind): string =>
  kind === 'Production' || kind === 'Organisation' || kind === 'Event' ? 'phase-2' : 'phase-3';

export const PLAN_RATING_OPTIONS: { value: PlanRating; label: string }[] = [
  { value: 1, label: 'Strongly oppose' },
  { value: 2, label: 'Oppose' },
  { value: 3, label: 'Neutral' },
  { value: 4, label: 'Support' },
  { value: 5, label: 'Strongly support' },
];

export function ratingLabel(rating: PlanRating | null): string {
  return PLAN_RATING_OPTIONS.find((o) => o.value === rating)?.label ?? 'Not rated';
}

export function allCriteriaRated(criteria: PlanCriterion[]): boolean {
  return criteria.every((c) => c.mine !== null);
}

/** web's card chip: Approved once the leader has passed, else Leading or Tied. */
export function planChip(p: PlanView): string | null {
  if (p.leader === 'Leading') return p.approved ? 'Approved' : 'Leading';
  if (p.leader === 'Tied') return 'Tied';
  return p.tally.status === 'Rejected' ? 'Rejected' : null;
}

/** "67% approved · 2 yes · 1 no" (web's plan footer). */
export function approvalLine(t: Pick<PlanTally, 'yes' | 'no'>): string {
  const total = t.yes + t.no;
  const percent = total === 0 ? 0 : Math.round((t.yes / total) * 100);
  return `${percent}% approved · ${t.yes} yes · ${t.no} no`;
}
