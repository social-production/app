/**
 * web's activity creation form and steps (utils/activityCreationSteps.ts). Times are the
 * browser's `datetime-local` values; places are plain labels until places land (M6).
 */
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
  place: string;
  meetingUrl: string;
  roles: RoleDraft[];
  planStage: string | null;
  note: string;
}

export type ActivityStepType =
  'title' | 'schedule' | 'location' | 'plan-stage' | 'roles' | 'note' | 'review';

export interface ActivityStep {
  type: ActivityStepType;
  question: string;
  helper: string;
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
    meetingUrl: '',
    roles: [emptyRole()],
    planStage: null,
    note: '',
  };
}

export function activitySteps(hasPlanStages: boolean): ActivityStep[] {
  const step = (type: ActivityStepType, question: string, helper: string): ActivityStep => ({
    type,
    question,
    helper,
  });
  return [
    step(
      'title',
      'What is this activity called?',
      'Give it a clear title so others know what they are signing up for.',
    ),
    step('schedule', 'When does it happen?', 'Set a start and finish time for this activity.'),
    step(
      'location',
      'Where does it happen?',
      'Search for a place or mark it online, like help requests and plans.',
    ),
    ...(hasPlanStages
      ? [
          step(
            'plan-stage',
            'Which plan stage does this support?',
            'Link the activity to a stage from the leading plan.',
          ),
        ]
      : []),
    step('roles', 'Who is needed?', 'Define the roles others can sign up for.'),
    step('note', 'What should happen?', 'Describe what participants will do during this activity.'),
    step('review', 'Review activity', 'Check the details before creating this activity.'),
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
  if (form.online) return form.place.trim() ? `Online · ${form.place.trim()}` : 'Online';
  return form.place.trim();
}

/** web's `activityScheduleValidationMessage`. */
export function scheduleProblem(form: ActivityForm, now = Date.now()): string | null {
  if (!form.scheduledAt.trim() || !form.endsAt.trim()) return null;
  const start = new Date(form.scheduledAt).getTime();
  const end = new Date(form.endsAt).getTime();
  if (Number.isNaN(start) || Number.isNaN(end)) return 'Choose valid start and finish times.';
  if (end <= start) return 'Finish must be after the start time.';
  if (start < now) return 'Start time must be in the future.';
  return null;
}

/** web's `validateActivityStep`. */
export function stepReady(type: ActivityStepType, form: ActivityForm): boolean {
  switch (type) {
    case 'title':
      return !!form.title.trim();
    case 'schedule':
      return !!form.scheduledAt && !!form.endsAt && scheduleProblem(form) === null;
    case 'location':
      return form.online || !!form.place.trim();
    case 'plan-stage':
      return !!form.planStage;
    case 'roles':
      return namedRoles(form.roles).length > 0;
    case 'note':
      return !!form.note.trim();
    case 'review':
      return true;
  }
}

/** web's `localDateTimeValue`: now, as a `datetime-local` minimum. */
export function localNow(date = new Date()): string {
  const pad = (n: number): string => String(n).padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}
