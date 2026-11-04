/**
 * Event plan schedules (ADR 0057), as web's `eventSchedule.ts` and web-backend
 * `utils/plan_schedule.py` keep them: typed in local time, stored as UTC dates and times,
 * shown in each reader's local time (the app has no display time zone setting yet, PARITY).
 */

/** project_integrity::EventSchedule. */
export interface EventSchedule {
  mode: 'any-day' | 'date' | 'range';
  start_date: string | null;
  end_date: string | null;
  start_time: string | null;
  finish_time: string | null;
}

/** web's plan wizard schedule fields (local dates and `HH:MM` times). */
export interface ScheduleForm {
  scheduleMode: 'date' | 'range';
  scheduledDate: string;
  rangeStartDate: string;
  rangeEndDate: string;
  startTime: string;
  finishTime: string;
}

export const emptyScheduleForm = (): ScheduleForm => ({
  scheduleMode: 'date',
  scheduledDate: '',
  rangeStartDate: '',
  rangeEndDate: '',
  startTime: '',
  finishTime: '',
});

const pad = (n: number): string => String(n).padStart(2, '0');
const utcDay = (d: Date): string =>
  `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`;
const utcTime = (d: Date): string => `${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}`;
const localDay = (d: Date): string =>
  `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;

/** A local date and time as a Date (`new Date('YYYY-MM-DDTHH:MM')` is local time). */
const local = (day: string, time: string): Date => new Date(`${day}T${time}`);

/** web's `schedule_to_utc`: the form's local dates and times as UTC. */
export function scheduleFromForm(f: ScheduleForm): EventSchedule {
  const range = f.scheduleMode === 'range';
  const startDay = range ? f.rangeStartDate : f.scheduledDate;
  const endDay = range ? f.rangeEndDate : f.scheduledDate;
  const start = local(startDay, f.startTime);
  const end = local(endDay, f.finishTime);
  return {
    mode: f.scheduleMode,
    start_date: utcDay(start),
    end_date: range ? utcDay(end) : null,
    start_time: utcTime(start),
    finish_time: utcTime(end),
  };
}

/** web's wizard gates: dates for the mode, both times, and the finish after the start. */
export function scheduleProblem(f: ScheduleForm): string | null {
  const range = f.scheduleMode === 'range';
  const startDay = range ? f.rangeStartDate : f.scheduledDate;
  const endDay = range ? f.rangeEndDate : f.scheduledDate;
  if (!startDay || !endDay) return range ? 'Choose a start and end date.' : 'Choose a date.';
  if (!f.startTime || !f.finishTime) return 'Choose a start and finish time.';
  if (local(endDay, f.finishTime) <= local(startDay, f.startTime))
    return 'The finish must be after the start.';
  return null;
}

/** The schedule's start and end instants (ms), or null when it names no date. */
export function scheduleBounds(s: EventSchedule): { start: number; end: number } | null {
  if (!s.start_date) return null;
  const end = s.end_date ?? s.start_date;
  const at = (day: string, time: string | null): number =>
    Date.parse(`${day}T${time ?? '00:00'}:00Z`);
  return { start: at(s.start_date, s.start_time), end: at(end, s.finish_time ?? s.start_time) };
}

/** The local days the schedule covers (web's plan days, where activities can be scheduled). */
export function scheduleDays(s: EventSchedule): string[] {
  const bounds = scheduleBounds(s);
  if (!bounds) return [];
  const days: string[] = [];
  const cursor = new Date(bounds.start);
  cursor.setHours(0, 0, 0, 0);
  while (cursor.getTime() <= bounds.end && days.length < 400) {
    days.push(localDay(cursor));
    cursor.setDate(cursor.getDate() + 1);
  }
  return days;
}

const localStamp = (d: Date): string =>
  `${localDay(d)}T${pad(d.getHours())}:${pad(d.getMinutes())}`;

/** web-backend's `_can_propose_event_activity`: the plan hasn't ended yet. */
export function planOpen(s: EventSchedule, now = Date.now()): boolean {
  const bounds = scheduleBounds(s);
  return !!bounds && bounds.end > now;
}

/** web's `eventScheduleDayBounds`: a plan day's window for the activity wizard (local
 * `datetime-local` values; the plan's start and finish times on its first and last day). */
export function dayWindow(s: EventSchedule, iso: string): { min: string; max: string } | null {
  const bounds = scheduleBounds(s);
  if (!bounds || !iso) return null;
  const [start, end] = [new Date(bounds.start), new Date(bounds.end)];
  if (iso < localDay(start) || iso > localDay(end)) return null;
  return {
    min: iso === localDay(start) ? localStamp(start) : `${iso}T00:00`,
    max: iso === localDay(end) ? localStamp(end) : `${iso}T23:59`,
  };
}

/** The wizard's default times moved inside a plan day's window (web opens the composer on
 * the plan day). */
export function withinWindow(
  times: { scheduledAt: string; endsAt: string },
  window: { min: string; max: string } | null,
): { scheduledAt: string; endsAt: string } {
  if (!window || (times.scheduledAt >= window.min && times.endsAt <= window.max)) return times;
  const start = new Date(window.min);
  const end = new Date(Math.min(start.getTime() + 3_600_000, new Date(window.max).getTime()));
  return { scheduledAt: window.min, endsAt: localStamp(end) };
}

const dateOptions: Intl.DateTimeFormatOptions = {
  weekday: 'short',
  month: 'short',
  day: 'numeric',
  year: 'numeric',
};
const timeOptions: Intl.DateTimeFormatOptions = { hour: 'numeric', minute: '2-digit' };

/** web's `formatLocalDateTimeRange` over the plan's start and end. */
export function scheduleLine(s: EventSchedule, zone: (d: Date) => string): string {
  const bounds = scheduleBounds(s);
  if (!bounds) return '';
  const start = new Date(bounds.start);
  const end = new Date(bounds.end);
  const suffix = (d: Date): string => (zone(d) ? ` ${zone(d)}` : '');
  const full = (d: Date): string =>
    `${d.toLocaleString(undefined, { ...dateOptions, ...timeOptions })}${suffix(d)}`;
  if (localDay(start) !== localDay(end)) return `${full(start)} – ${full(end)}`;
  const day = start.toLocaleDateString(undefined, dateOptions);
  const from = start.toLocaleTimeString(undefined, timeOptions);
  const to = end.toLocaleTimeString(undefined, timeOptions);
  return `${day}, ${from} – ${to}${suffix(end)}`;
}

/** The wizard's review line: "2026-11-14 · 09:00 – 13:00" (web's `scheduleSummary`). */
export function scheduleSummary(f: ScheduleForm): string {
  const days =
    f.scheduleMode === 'range'
      ? `${f.rangeStartDate || '—'} to ${f.rangeEndDate || '—'}`
      : f.scheduledDate || '—';
  return `${days} · ${f.startTime || '—'} – ${f.finishTime || '—'}`;
}

const PLACEHOLDER_LABELS = new Set(['tbd', 'not specified', 'to be determined']);

/** web's `isImplementedScheduleLabel`: a time or place label that isn't a placeholder. */
export function implementedLabel(label: string | null | undefined): string {
  const s = (label ?? '').trim();
  return s && !PLACEHOLDER_LABELS.has(s.toLowerCase()) ? s : '';
}
