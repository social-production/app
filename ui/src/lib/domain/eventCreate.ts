/**
 * web's CreateEventPage rules (ADR 0057, 0061): which steps each audience shows, and the up-front
 * plan an organizer-controlled private event starts with (web's `validatePrivatePlan`).
 */

import { scheduleBounds, scheduleFromForm } from './eventSchedule.ts';
import type { PlanForm } from './planForm.ts';

export type EventStepId =
  'basics' | 'audience' | 'scope' | 'control' | 'location' | 'plan' | 'overview';

export interface EventStep {
  id: EventStepId;
  title: string;
}

/** web's three wizard step lists: public, private, and private organizer-controlled. */
export function eventSteps(isPrivate: boolean, upfrontPlan: boolean): EventStep[] {
  if (!isPrivate) {
    return [
      { id: 'basics', title: 'Basics' },
      { id: 'audience', title: 'Audience' },
      { id: 'scope', title: 'Tags' },
      { id: 'location', title: 'Location' },
      { id: 'overview', title: 'Overview' },
    ];
  }
  return [
    { id: 'basics', title: 'Basics' },
    { id: 'audience', title: 'Audience' },
    { id: 'scope', title: 'Who' },
    { id: 'control', title: 'Control' },
    { id: 'location', title: 'Location' },
    ...(upfrontPlan ? [{ id: 'plan', title: 'Plan' } as EventStep] : []),
    { id: 'overview', title: 'Overview' },
  ];
}

/** web's `validatePrivatePlan` messages, in its order (an empty list means ready). */
export function upfrontPlanProblems(f: PlanForm, physical: boolean, now = Date.now()): string[] {
  const out: string[] = [];
  const range = f.scheduleMode === 'range';
  const hasDates = range ? !!f.rangeStartDate && !!f.rangeEndDate : !!f.scheduledDate;
  if (!f.title.trim()) out.push('Add a plan title.');
  if (!f.description.trim()) out.push('Add a plan description.');
  if (!f.location.trim() && physical) out.push('Add a location for the event plan.');
  if (!range && !f.scheduledDate) out.push('Choose the event date.');
  if (range && !hasDates) out.push('Choose both the start date and end date.');
  if (!f.startTime) out.push('Choose a start time.');
  if (!f.finishTime) out.push('Choose a finish time.');
  if (hasDates && f.startTime && f.finishTime) {
    const bounds = scheduleBounds(scheduleFromForm(f));
    if (!bounds || bounds.end <= bounds.start) out.push('Finish must be after the event start.');
    else if (bounds.start <= now) out.push('Event plans cannot start in the past.');
  }
  const done = (s: PlanForm['stages'][number]) => !!s.title.trim() && !!s.details.trim();
  const started = (s: PlanForm['stages'][number]) => !!s.title.trim() || !!s.details.trim();
  if (!f.stages.some(done)) out.push('Add at least one stage with both a title and details.');
  else if (f.stages.some((s) => started(s) && !done(s))) out.push('Finish every stage you start.');
  return out;
}

/** The demand note the up-front plan carries: web leaves it empty, plan integrity needs one
 * (ADR 0061). */
export const ORGANIZER_DEMAND_NOTE = 'Set by the organizers; this event takes no support signals.';
