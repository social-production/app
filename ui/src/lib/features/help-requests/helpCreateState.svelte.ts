import {
  endsAfter,
  hoursAfter,
  inputMicros,
  scheduleLabel,
  type HelpRoleDraft,
} from '#lib/domain/helpRequests.ts';
import type { LocationChoice } from '#lib/domain/projects.ts';
import { explain } from '#lib/services/errors.ts';
import { createHelpRequest } from '#lib/services/helpRequests.ts';

export const HELP_STEPS = [
  { id: 'basics', title: 'Basics' },
  { id: 'location', title: 'Location' },
  { id: 'timing', title: 'Timing' },
  { id: 'scope', title: 'Scope' },
  { id: 'roles', title: 'Roles' },
  { id: 'overview', title: 'Overview' },
] as const;

type StepId = (typeof HELP_STEPS)[number]['id'];

/** web's CreateHelpRequestPage draft: basics, location, timing, tags, roles (ADR 0067). */
export class HelpCreateState {
  step = $state(0);
  title = $state('');
  body = $state('');
  intent = $state<LocationChoice>('physical');
  place = $state('');
  neededAt = $state(hoursAfter(null, 2));
  endsAt = $state('');
  tags = $state<string[]>([]);
  roles = $state<HelpRoleDraft[]>([{ label: '', description: '', slots: 1 }]);
  submitting = $state(false);
  message = $state<string>();

  constructor() {
    this.endsAt = hoursAfter(this.neededAt, 2);
  }

  readonly stepId = $derived<StepId>(HELP_STEPS[this.step]?.id ?? 'basics');
  readonly namedRoles = $derived(this.roles.filter((r) => r.label.trim()));
  readonly locationLabel = $derived(
    this.intent === 'online'
      ? 'Online'
      : this.intent === 'physical'
        ? this.place.trim() || 'Add a location'
        : 'Choose physical or online',
  );
  readonly schedule = $derived(scheduleLabel(this.neededAt, this.endsAt));

  /** web's buildValidationMessages, per step (web needs a channel tag; the app has none yet). */
  readonly problems = $derived.by(() => ({
    basics: [
      !this.title.trim() && 'Help request title is required.',
      !this.body.trim() && 'Description is required.',
    ],
    location: [
      this.intent === 'later' && 'Choose a physical or online location before publishing.',
      this.intent === 'physical' && !this.place.trim() && 'Location is required.',
    ],
    timing: [
      !this.neededAt && 'When you need help is required.',
      !this.endsAt && 'An end time is required.',
      this.neededAt &&
        this.endsAt &&
        !endsAfter(this.neededAt, this.endsAt) &&
        'End time must be after the start time.',
    ],
    roles: [this.namedRoles.length === 0 && 'Add at least one role (for example, Driver).'],
  }));
  readonly missing = $derived(
    ((this.problems as Record<string, (string | false | '')[]>)[this.stepId] ?? []).filter(
      (m): m is string => !!m,
    ),
  );
  readonly allProblems = $derived(
    Object.values(this.problems)
      .flat()
      .filter((m): m is string => !!m),
  );

  setNeededAt(value: string): void {
    this.neededAt = value;
    if (!this.endsAt || !endsAfter(value, this.endsAt)) this.endsAt = hoursAfter(value, 2);
  }

  addRole(): void {
    this.roles = [...this.roles, { label: '', description: '', slots: 1 }];
  }

  removeRole(index: number): void {
    if (this.roles.length > 1) this.roles = this.roles.filter((_, i) => i !== index);
  }

  /** The new request's listing id, or undefined with `message` set. */
  async submit(): Promise<string | undefined> {
    if (this.allProblems.length > 0) {
      this.message = this.allProblems[0];
      return undefined;
    }
    this.submitting = true;
    this.message = undefined;
    try {
      return await createHelpRequest({
        title: this.title.trim(),
        body: this.body.trim(),
        online: this.intent === 'online',
        location: this.intent === 'online' ? 'Online' : this.place.trim(),
        startsAt: inputMicros(this.neededAt),
        endsAt: inputMicros(this.endsAt),
        tags: this.tags,
        roles: this.namedRoles.map((r) => ({ ...r, label: r.label.trim() })),
      });
    } catch (e) {
      this.message = explain(e);
      return undefined;
    } finally {
      this.submitting = false;
    }
  }
}
