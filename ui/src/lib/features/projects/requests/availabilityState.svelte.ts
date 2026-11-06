import { minutesOf, type AvailabilityRule } from '#lib/domain/availability.ts';
import { preferredTime } from '#lib/domain/serviceRequests.ts';
import {
  addAvailabilityRule,
  createActivity,
  listAvailabilityRules,
  removeAvailabilityRule,
} from '#lib/services/activities.ts';
import { explain } from '#lib/services/errors.ts';

const micros = (local: string): number => Date.parse(local) * 1000;

/** The browser's IANA zone (web's `getDisplayTimezone` until the app has that setting). */
export const browserZone = (): string => Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';

/**
 * A personal service's listed availability (web's "Add availability" sheet, ADR 0077): one-off
 * slots are activities, weekly hours are rules. Only the service's creator writes either.
 */
export class AvailabilityState {
  rules = $state<AvailabilityRule[]>([]);
  error = $state('');
  busy = $state(false);
  open = $state(false);
  kind = $state<'one-off' | 'weekly'>('one-off');
  oneOff = $state({ scheduledAt: '', endsAt: '' });
  // web's weekly defaults: Tuesday 09:00–12:00.
  weekly = $state({ weekday: 1, start: '09:00', end: '12:00', note: '' });

  constructor(private readonly project: string) {}

  async load(): Promise<void> {
    try {
      this.rules = await listAvailabilityRules(this.project);
    } catch (e) {
      this.error = explain(e);
    }
  }

  /** web's `openCalendarComposerForDay` for the creator: that day 18:00–19:00. */
  openSheet(day: string): void {
    this.kind = 'one-off';
    this.oneOff = day ? { scheduledAt: `${day}T18:00`, endsAt: `${day}T19:00` } : preferredTime();
    this.error = '';
    this.open = true;
  }

  /** web's personal `submitActivity`: an "Available" activity with a Service lead role. */
  async addOneOff(location: string): Promise<boolean> {
    const { scheduledAt, endsAt } = this.oneOff;
    if (!scheduledAt || !endsAt || Date.parse(endsAt) <= Date.parse(scheduledAt)) {
      this.error = 'Add a start and a later finish time.';
      return false;
    }
    return this.run(() =>
      createActivity(this.project, {
        title: 'Available',
        startsAt: micros(scheduledAt),
        endsAt: micros(endsAt),
        online: !location,
        place: location || 'Online',
        meetingUrl: null,
        note: 'Availability shared by the service creator.',
        planStage: null,
        roles: [{ label: 'Service lead', required: 1, maximum: null, suggest: null }],
      }).then(() => undefined),
    );
  }

  /** web's `submitWeeklyRule`. */
  async addWeekly(): Promise<boolean> {
    const w = this.weekly;
    const start = minutesOf(w.start);
    const end = minutesOf(w.end);
    if (start === null || end === null || end <= start) return false;
    const ok = await this.run(() =>
      addAvailabilityRule(this.project, {
        weekday: w.weekday,
        start,
        end,
        zone: browserZone(),
        note: w.note.trim(),
      }),
    );
    if (ok) this.weekly = { ...w, note: '' };
    return ok;
  }

  async remove(rule: string): Promise<void> {
    await this.run(() => removeAvailabilityRule(this.project, rule));
  }

  private async run(work: () => Promise<void>): Promise<boolean> {
    this.busy = true;
    this.error = '';
    try {
      await work();
      this.open = false;
      await this.load();
      return true;
    } catch (e) {
      this.error = explain(e);
      return false;
    } finally {
      this.busy = false;
    }
  }
}
