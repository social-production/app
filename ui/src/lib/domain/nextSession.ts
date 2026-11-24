/**
 * A feed card's next planned activity (owner, 2026-11-24: "all content that has a next
 * activity planned should show" it, like the Meetup mockup's card). Times are Holochain
 * microseconds, shown in the viewer's chosen zone.
 */

import { locale, plural, t } from '#lib/i18n/index.ts';

import { displayZone } from './zone.ts';

export interface NextActivity {
  id: string;
  title: string;
  startsAt: number;
  endsAt: number;
  committed: number;
  minimum: number;
  /** Activities after this one that haven't ended. */
  more: number;
}

const DAY_MS = 86_400_000;

/** The calendar day of `ms` in the display zone, as a sortable "YYYY-MM-DD". */
function dayOf(ms: number): string {
  return new Date(ms).toLocaleDateString('en-CA', { timeZone: displayZone() });
}

/** "Now", "Today 6:00 pm", "Tomorrow 6:00 pm", "Thu 6:00 pm" within a week, else "Sat 6 Dec, 11:00 am". */
export function nextWhen(next: NextActivity, nowMs = Date.now()): string {
  const start = next.startsAt / 1000;
  if (start <= nowMs && next.endsAt / 1000 > nowMs) return t('Now');
  const zone = displayZone();
  const time = new Date(start).toLocaleTimeString(locale(), {
    hour: 'numeric',
    minute: '2-digit',
    timeZone: zone,
  });
  const day = dayOf(start);
  if (day === dayOf(nowMs)) return t('Today {time}', { time });
  if (day === dayOf(nowMs + DAY_MS)) return t('Tomorrow {time}', { time });
  if (start - nowMs < 6 * DAY_MS) {
    const weekday = new Date(start).toLocaleDateString(locale(), {
      weekday: 'short',
      timeZone: zone,
    });
    return `${weekday} ${time}`;
  }
  const date = new Date(start).toLocaleDateString(locale(), {
    weekday: 'short',
    day: 'numeric',
    month: 'short',
    timeZone: zone,
  });
  return `${date}, ${time}`;
}

/** "2 of 4 going", "3 going", or nothing when nobody has a role yet and none is needed. */
export function nextGoing(next: NextActivity): string {
  if (next.minimum > 0) return t('{n} of {of} going', { n: next.committed, of: next.minimum });
  return next.committed > 0 ? t('{n} going', { n: next.committed }) : '';
}

/** "+2 more" sessions after this one. */
export const nextMore = (next: NextActivity): string =>
  next.more > 0 ? plural(next.more, '+{n} more', '+{n} more') : '';
