/**
 * Project plans (M3b, ADR 0038). Mirrors the governance zome's plan board; wording and wizard
 * steps are web's (web/src/lib/utils/planRubric.ts, ProjectLifecyclePlanPhaseContent.svelte).
 */

import type { ProposalStatus, VoteChoice } from './governance.ts';
import type { ProjectMode, ProjectSubtype } from './projects.ts';

/** sp_governance::plans::PlanKind ("Organisation" is web's operations plan). */
export type PlanKind = 'Production' | 'Distribution' | 'Organisation' | 'Access';
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
  recipients: string | null;
  stages: PlanStage[];
}

export interface PlanCriterion {
  id: string;
  label: string;
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

/** The plan kind a phase of this mode takes (sp_governance `PlanKind::for_phase`). */
export function planKindFor(mode: ProjectMode, phase: string): PlanKind | null {
  const collective = mode === 'collective-service';
  if (mode === 'personal-service') return null;
  if (phase === 'phase-2') return collective ? 'Organisation' : 'Production';
  if (phase === 'phase-3') return collective ? 'Access' : 'Distribution';
  return null;
}

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

export type PlanStepType =
  | 'plan-overview'
  | 'subtype'
  | 'repository'
  | 'location'
  | 'distribution-location'
  | 'demand-note'
  | 'stages'
  | 'review';

export interface PlanStep {
  id: PlanStepType;
  question: string;
  helper?: string;
}

export interface PlanForm {
  title: string;
  description: string;
  demandNote: string;
  subtype: ProjectSubtype;
  repositoryUrl: string;
  location: string;
  sameAsProduction: boolean;
  recipients: string;
  stages: { title: string; details: string; startDate: string; endDate: string }[];
}

export function emptyPlanForm(subtype: ProjectSubtype): PlanForm {
  return {
    title: '',
    description: '',
    demandNote: '',
    subtype,
    repositoryUrl: '',
    location: '',
    sameAsProduction: false,
    recipients: '',
    stages: [{ title: '', details: '', startDate: '', endDate: '' }],
  };
}

const REVIEW: PlanStep = {
  id: 'review',
  question: 'Review your plan before submitting',
  helper: 'Check each section, then submit when you are ready.',
};

/** web's `buildProjectProductionCreationSteps` / `buildProjectDistributionCreationSteps`. */
export function planSteps(kind: PlanKind, subtype: ProjectSubtype): PlanStep[] {
  const producing = kind === 'Production' || kind === 'Organisation';
  const steps: PlanStep[] = [
    {
      id: 'plan-overview',
      question: 'What is this plan called, and what does it deliver?',
      helper: producing
        ? 'Use a clear title and explain what will be produced and why this approach makes sense.'
        : 'Use a clear title and explain how people will access or receive what is produced.',
    },
  ];
  if (producing) {
    steps.push({ id: 'subtype', question: 'What type of production plan is this?' });
    if (subtype === 'software')
      steps.push({
        id: 'repository',
        question: 'What is the official repository URL?',
        helper:
          'Link a public repository under AGPL v3. Assessors need a place they can verify, and the license keeps the software in the commons.',
      });
    else
      steps.push({
        id: 'location',
        question: 'Where will production or operations happen?',
        helper: 'Choose the primary project or production site.',
      });
  } else {
    steps.push({
      id: 'distribution-location',
      question: 'Where will distribution or access happen?',
      helper: 'This can be the same address as the production plan location.',
    });
  }
  steps.push(
    {
      id: 'demand-note',
      question: 'How does this plan respond to current support?',
      helper:
        'Support signals show how many people want this to exist right now. Assessors compare every plan against that need, so say whether your plan meets it and what gap remains.',
    },
    {
      id: 'stages',
      question: 'What are the stages of this plan?',
      helper:
        'Break the work into stages. Each stage needs a title and details. You can add, remove, and reorder stages.',
    },
    REVIEW,
  );
  return steps;
}

/** web's `validateStep`: whether Next is allowed. */
export function stepReady(step: PlanStepType, f: PlanForm): boolean {
  switch (step) {
    case 'plan-overview':
      return !!f.title.trim() && !!f.description.trim();
    case 'demand-note':
      return !!f.demandNote.trim();
    case 'repository':
      return !!f.repositoryUrl.trim();
    case 'location':
    case 'distribution-location':
      return !!f.location.trim();
    case 'stages':
      return f.stages.length > 0 && f.stages.every((s) => s.title.trim() && s.details.trim());
    default:
      return true;
  }
}

const orNull = (text: string): string | null => (text.trim() ? text.trim() : null);

/** The form as the zome takes it. The subtype only matters for production/operations plans. */
export function planFromForm(kind: PlanKind, f: PlanForm): Plan {
  const producing = kind === 'Production' || kind === 'Organisation';
  const software = producing && f.subtype === 'software';
  return {
    kind,
    title: f.title.trim(),
    description: f.description.trim(),
    demand_note: f.demandNote.trim(),
    subtype: producing ? f.subtype : null,
    repository_url: software ? orNull(f.repositoryUrl) : null,
    location: software ? null : orNull(f.location),
    recipients: producing ? null : orNull(f.recipients),
    stages: f.stages.map((s) => ({
      title: s.title.trim(),
      details: s.details.trim(),
      start_date: orNull(s.startDate),
      end_date: orNull(s.endDate),
    })),
  };
}
