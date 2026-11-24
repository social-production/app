/**
 * The plan wizard's form and steps (web's PlanCreationWizard and planRubric.ts step builders),
 * for project plans and event plans (ADR 0057).
 */

import { t } from '#lib/i18n/index.ts';
import {
  emptyScheduleForm,
  scheduleFromForm,
  scheduleProblem,
  type ScheduleForm,
} from './eventSchedule.ts';
import {
  emptyPlaceValue,
  fitLabel,
  fromPlace,
  placeRecord,
  type Place,
  type PlaceValue,
} from './places.ts';
import type { Plan, PlanKind } from './plans.ts';
import { LOCATION_LABEL_MAX, type ProjectSubtype, type RequestMode } from './projects.ts';

export type PlanStepType =
  | 'plan-overview'
  | 'subtype'
  | 'repository'
  | 'location'
  | 'distribution-location'
  | 'requests'
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

/** The leading production plan's place, for "Same as production plan location". */
export interface ProductionLocation {
  label: string;
  place: Place | null;
}

export interface PlanForm extends ScheduleForm {
  title: string;
  description: string;
  demandNote: string;
  subtype: ProjectSubtype;
  repositoryUrl: string;
  location: string;
  /** The picker's value behind `location` (ADR 0080). */
  place: PlaceValue;
  sameAsProduction: boolean;
  recipients: string;
  /** An operations plan's request mode; empty takes no requests (ADR 0059). */
  requests: RequestMode | '';
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
    place: emptyPlaceValue(),
    sameAsProduction: false,
    recipients: '',
    requests: '',
    stages: [{ title: '', details: '', startDate: '', endDate: '' }],
    ...emptyScheduleForm(),
  };
}

const REVIEW: PlanStep = {
  id: 'review',
  question: t('Check it, then send it to a vote'),
};

const DEMAND_NOTE: PlanStep = {
  id: 'demand-note',
  question: t('What support does it answer?'),
  helper: t('Say how this plan meets the support people have shown, and what gap remains.'),
};

const STAGES: PlanStep = {
  id: 'stages',
  question: t('What are the stages?'),
  helper: t('Each stage gets a title and what happens in it. Hold ⠿ to reorder.'),
};

/** The date and time steps of a dated schedule. */
function datedSteps(f: ScheduleForm): PlanStep[] {
  return [
    f.scheduleMode === 'range'
      ? { id: 'schedule-range', question: t('What date range should the event cover?') }
      : { id: 'schedule-date', question: t('Which date should the event use?') },
    {
      id: 'schedule-time',
      question: t('What are the start and finish times?'),
      helper: t(
        'Marked plan days become the days where activity can be scheduled from the calendar.',
      ),
    },
  ];
}

/** web's `buildEventPlanCreationSteps`, with only the date step its schedule mode shows; no
 * set date skips the dates and times (sessions are the activities). */
function eventPlanSteps(f: ScheduleForm): PlanStep[] {
  if (isMeetupForm(f)) return meetupPlanSteps();
  return [
    {
      id: 'plan-overview',
      question: t('What is this plan called, and what does it do?'),
      helper: t('A clear title, and what will happen.'),
    },
    {
      id: 'schedule-mode',
      question: t('When is it?'),
      helper: t('One day, or several days in a row.'),
    },
    ...datedSteps(f),
    { id: 'location', question: t('Where will this event happen?') },
    DEMAND_NOTE,
    STAGES,
    REVIEW,
  ];
}

/** A meetup plan (ADR 0119): an event plan with a usual day or none. */
export const isMeetupForm = (f: ScheduleForm): boolean =>
  f.scheduleMode === 'weekly' || f.scheduleMode === 'any-day';

function meetupPlanSteps(): PlanStep[] {
  return [
    {
      id: 'plan-overview',
      question: t('What is this meetup called, and what happens?'),
      helper: t('A clear title, and what people do when they meet.'),
    },
    {
      id: 'schedule-mode',
      question: t('How do you meet?'),
      helper: t(
        'Members add each session as an activity, and people sign up to the ones they come to.',
      ),
    },
    { id: 'location', question: t('Where do you meet?') },
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
      question: t('What is this plan called, and what does it deliver?'),
      helper: producing
        ? t('A clear title, and what will be made and how.')
        : t('A clear title, and how people will get what is made.'),
    },
  ];
  if (producing) {
    steps.push({ id: 'subtype', question: t('What type of production plan is this?') });
    if (subtype === 'software')
      steps.push({
        id: 'repository',
        question: t('What is the official repository URL?'),
        helper: t(
          'Link a public repository under AGPL v3. Assessors need a place they can verify, and the license keeps the software in the commons.',
        ),
      });
    else
      steps.push({
        id: 'location',
        question: t('Where will production or operations happen?'),
        helper: t('Choose the primary project or production site.'),
      });
    if (kind === 'Organisation')
      steps.push({
        id: 'requests',
        question: t('Will this service take requests?'),
        helper: t(
          'People outside the service can ask it for something. Members can change this later by a vote.',
        ),
      });
  } else {
    steps.push({
      id: 'distribution-location',
      question: t('Where will distribution or access happen?'),
      helper: t('This can be the same address as the production plan location.'),
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
  const location = software ? null : orNull(fitLabel(f.location, LOCATION_LABEL_MAX));
  return {
    kind,
    title: f.title.trim(),
    description: f.description.trim(),
    demand_note: f.demandNote.trim(),
    subtype: producing ? f.subtype : null,
    repository_url: software ? orNull(f.repositoryUrl) : null,
    location,
    place: location ? placeRecord(f.place, location) : null,
    recipients: producing ? null : orNull(f.recipients),
    stages: f.stages.map((s) => ({
      title: s.title.trim(),
      details: s.details.trim(),
      start_date: orNull(s.startDate),
      end_date: orNull(s.endDate),
    })),
    ...(event ? { schedule: scheduleFromForm(f) } : {}),
    ...(kind === 'Organisation' && f.requests ? { requests: f.requests } : {}),
  };
}

/** web's handleLocationChange (`resolvedPickerLabel`: an online place is "Online"). */
export function withPlanPlace(f: PlanForm, value: PlaceValue): PlanForm {
  const online = value.mode === 'online' || value.isOnline;
  const label = online ? value.displayLabel.trim() || 'Online' : value.displayLabel;
  return { ...f, location: label, place: value };
}

/** web's applySameAsProductionLocation / clearSameAsProductionLocation. */
export function sameAsProduction(
  f: PlanForm,
  checked: boolean,
  production: ProductionLocation | null,
): PlanForm {
  const label = checked ? (production?.label ?? '') : '';
  const place = checked && production ? fromPlace(production.place, label) : emptyPlaceValue();
  return { ...f, sameAsProduction: checked, location: label, place };
}
