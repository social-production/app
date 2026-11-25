/**
 * web's activity creation form and steps (utils/activityCreationSteps.ts). Times are the
 * browser's `datetime-local` values; the place comes from web's LocationPicker (ADR 0080).
 */
import { t } from '#lib/i18n/index.ts';
import {
  dedupeQuickPicks,
  emptyPlaceValue,
  quickPick,
  type Place,
  type PlaceQuickPick,
  type PlaceValue,
} from './places.ts';
import { wall, wallMicros } from './zone.ts';
import type { PlanView } from './plans.ts';

export interface RoleDraft {
  label: string;
  required: number;
  maximum: number | null;
  /** Suggested person (agent key) and the name shown for them. */
  suggest: { agent: string; name: string } | null;
}

export interface ActivityForm {
  title: string;
  scheduledAt: string;
  endsAt: string;
  online: boolean;
  /** The picked place's label ("Online" for an online activity). */
  place: string;
  /** The picker's value behind `online` and `place`. */
  location: PlaceValue;
  meetingUrl: string;
  roles: RoleDraft[];
  planStage: string | null;
  note: string;
}

/** The four steps (Activity creation mockup, owner 2026-11-25): what and when, where (and
 * which plan stage), who's needed, then check and create with an optional note. */
export type ActivityStepType = 'when' | 'where' | 'who' | 'review';

export interface ActivityStep {
  type: ActivityStepType;
  question: string;
}

export const emptyRole = (): RoleDraft => ({
  label: '',
  required: 1,
  maximum: null,
  suggest: null,
});

export function newActivityForm(place = ''): ActivityForm {
  return {
    title: '',
    scheduledAt: '',
    endsAt: '',
    online: false,
    place,
    location: { ...emptyPlaceValue(), displayLabel: place },
    meetingUrl: '',
    // One role to start, so a session anyone can join needs no setup (Activity mockup §3).
    roles: [{ ...emptyRole(), label: t('Players') }],
    planStage: null,
    note: '',
  };
}

export function activitySteps(): ActivityStep[] {
  return [
    { type: 'when', question: t("What's on, and when?") },
    { type: 'where', question: t('Where?') },
    { type: 'who', question: t("Who's needed?") },
    { type: 'review', question: t('Check and create') },
  ];
}

/** Roles with a name, counts made sane (web's `normalizedRoleRequirements`). */
export function namedRoles(roles: RoleDraft[]): RoleDraft[] {
  return roles
    .map((r) => {
      const required = Math.max(1, Math.floor(Number(r.required)) || 1);
      const max = Number(r.maximum);
      const maximum =
        r.maximum !== null && Number.isFinite(max) ? Math.max(required, Math.floor(max)) : null;
      return { ...r, label: r.label.trim(), required, maximum };
    })
    .filter((r) => r.label);
}

export const minimumPeople = (roles: RoleDraft[]): number =>
  roles.reduce((n, r) => n + Math.max(1, Number(r.required) || 1), 0);

/** web's place summary: "Online", "Online · Zoom" or the place. */
export function placeSummary(form: ActivityForm): string {
  const place = form.place.trim();
  if (form.online) return place && place !== 'Online' ? `${t('Online')} · ${place}` : t('Online');
  return place;
}

/** web's syncFormFromLocationValue. */
export function withLocation(form: ActivityForm, value: PlaceValue): ActivityForm {
  const online = value.mode === 'online' || value.isOnline;
  return { ...form, location: value, online, place: value.displayLabel };
}

/** web's `buildActivityLocationQuickPicks` sources per lifecycle: the record's own place, then
 * the approved leading plans' (EventActivityPhase, ProductiveLifecyclePhaseFive, and the
 * generic and collective-service phase five). */
export function activityQuickPicks(
  mode: string,
  own: { label: string; place: Place | null | undefined },
  plans: PlanView[],
): PlaceQuickPick[] {
  const leading = (kinds: string[]) =>
    plans.find((p) => kinds.includes(p.plan.kind) && p.leader === 'Leading' && p.approved)?.plan;
  const pick = (id: string, sourceLabel: string, plan: PlanView['plan'] | undefined) =>
    plan && quickPick({ id, sourceLabel, label: plan.location ?? '', place: plan.place ?? null });
  if (mode === 'event') {
    return dedupeQuickPicks([
      quickPick({ id: 'event-initial', sourceLabel: 'Event location', ...own }),
      pick('event-plan', 'Plan location', leading(['Event'])),
    ]);
  }
  return dedupeQuickPicks([
    quickPick({ id: 'project-initial', sourceLabel: 'Project location', ...own }),
    pick(
      'production-plan',
      mode === 'productive' ? t('Production plan') : t('Plan location'),
      leading(['Production', 'Organisation', 'Access']),
    ),
    pick('distribution-plan', 'Distribution plan', leading(['Distribution'])),
  ]);
}

/** web's `activityScheduleValidationMessage`. */
export function scheduleProblem(form: ActivityForm, now = Date.now()): string | null {
  if (!form.scheduledAt.trim() || !form.endsAt.trim()) return null;
  const start = wallMicros(form.scheduledAt) / 1000;
  const end = wallMicros(form.endsAt) / 1000;
  if (Number.isNaN(start) || Number.isNaN(end)) return t('Choose valid start and finish times.');
  if (end <= start) return t('Finish must be after the start time.');
  if (start < now) return t('Start time must be in the future.');
  return null;
}

/** web's `validateActivityStep`, for the four steps; the stage is asked on Where. */
export function stepReady(type: ActivityStepType, form: ActivityForm, hasStages = false): boolean {
  switch (type) {
    case 'when':
      return (
        !!form.title.trim() && !!form.scheduledAt && !!form.endsAt && scheduleProblem(form) === null
      );
    case 'where':
      return (form.online || !!form.place.trim()) && (!hasStages || !!form.planStage);
    case 'who':
      return namedRoles(form.roles).length > 0;
    case 'review':
      return true;
  }
}

/** web's `localDateTimeValue`: now, as a `datetime-local` minimum. */
export function localNow(date = wall(Date.now())): string {
  const pad = (n: number): string => String(n).padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}
