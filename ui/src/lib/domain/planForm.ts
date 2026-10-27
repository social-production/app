/**
 * The plan wizard's form and steps (web's PlanCreationWizard and planRubric.ts step builders),
 * for project plans and event plans (ADR 0057).
 */

import {
  emptyScheduleForm,
  scheduleFromForm,
  scheduleProblem,
  type ScheduleForm,
} from './eventSchedule.ts';
import type { Plan, PlanKind } from './plans.ts';
import type { ProjectSubtype } from './projects.ts';

export type PlanStepType =
  | 'plan-overview'
  | 'subtype'
  | 'repository'
  | 'location'
  | 'distribution-location'
  | 'demand-note'
  | 'schedule-mode'
  | 'schedule-date'
  | 'schedule-range'
  | 'schedule-time'
  | 'stages'
  | 'review';

export interface PlanStep {
  id: PlanStepType;
  question: string;
  helper?: string;
}

export interface PlanForm extends ScheduleForm {
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
    ...emptyScheduleForm(),
  };
}

const REVIEW: PlanStep = {
  id: 'review',
  question: 'Review your plan before submitting',
  helper: 'Check each section, then submit when you are ready.',
};

const DEMAND_NOTE: PlanStep = {
  id: 'demand-note',
  question: 'How does this plan respond to current support?',
  helper:
    'Support signals show how many people want this to exist right now. Assessors compare every plan against that need, so say whether your plan meets it and what gap remains.',
};

const STAGES: PlanStep = {
  id: 'stages',
  question: 'What are the stages of this plan?',
  helper:
    'Break the work into stages. Each stage needs a title and details. You can add, remove, and reorder stages.',
};

/** web's `buildEventPlanCreationSteps`, with only the date step its schedule mode shows. */
function eventPlanSteps(f: ScheduleForm): PlanStep[] {
  return [
    {
      id: 'plan-overview',
      question: 'What is this plan called, and what does it do?',
      helper:
        'Use a clear title and explain what will happen and why this plan is worth considering.',
    },
    {
      id: 'schedule-mode',
      question: 'How should the event be scheduled?',
      helper: 'Choose a single date or a date range.',
    },
    f.scheduleMode === 'range'
      ? { id: 'schedule-range', question: 'What date range should the event cover?' }
      : { id: 'schedule-date', question: 'Which date should the event use?' },
    {
      id: 'schedule-time',
      question: 'What are the start and finish times?',
      helper: 'Marked plan days become the days where activity can be scheduled from the calendar.',
    },
    { id: 'location', question: 'Where will this event happen?' },
    DEMAND_NOTE,
    STAGES,
    REVIEW,
  ];
}

/** web's `buildProjectProductionCreationSteps` / `buildProjectDistributionCreationSteps`, and
 * `buildEventPlanCreationSteps` for events. */
export function planSteps(kind: PlanKind, f: PlanForm): PlanStep[] {
  if (kind === 'Event') return eventPlanSteps(f);
  const subtype = f.subtype;
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
  steps.push(DEMAND_NOTE, STAGES, REVIEW);
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
    case 'schedule-date':
      return !!f.scheduledDate;
    case 'schedule-range':
      return !!f.rangeStartDate && !!f.rangeEndDate && f.rangeEndDate >= f.rangeStartDate;
    case 'schedule-time':
      return scheduleProblem(f) === null;
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
  const event = kind === 'Event';
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
    ...(event ? { schedule: scheduleFromForm(f) } : {}),
  };
}
