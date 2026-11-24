import { locale, plural, t } from '#lib/i18n/index.ts';

import type { PlanForm } from './planForm.ts';

/**
 * Words for the plan wizard's stage cards and review card (Edit, Plan and Meetup mockups §B,
 * owner 2026-11-24): "Dec 6 → Feb 28" and "3 · Dec 6 → Feb 28".
 */

export type PlanStage = PlanForm['stages'][number];

export function emptyStage(): PlanStage {
  return { title: '', details: '', startDate: '', endDate: '' };
}

/** A `yyyy-mm-dd` field as "Dec 6", or "" when unset or unreadable. */
export function shortDay(value: string): string {
  const [y, m, d] = value.split('-').map(Number);
  if (!y || !m || !d) return '';
  return new Date(y, m - 1, d).toLocaleDateString(locale(), { month: 'short', day: 'numeric' });
}

/** "Dec 6 → Feb 28", "From Dec 6", "Until Feb 28" or "". */
export function dayRange(start: string, end: string): string {
  const [a, b] = [shortDay(start), shortDay(end)];
  if (a && b) return a === b ? a : `${a} → ${b}`;
  if (a) return t('From {day}', { day: a });
  if (b) return t('Until {day}', { day: b });
  return '';
}

/** The small line under a stage card's title: its details, then its dates. */
export function stageLine(stage: PlanStage): string {
  return [stage.details.trim(), dayRange(stage.startDate, stage.endDate)]
    .filter(Boolean)
    .join(' · ');
}

/** The review card's Stages row: "3 · Dec 6 → Feb 28" across every stage's dates. */
export function stagesSummary(stages: PlanStage[]): string {
  const starts = stages
    .map((s) => s.startDate || s.endDate)
    .filter(Boolean)
    .sort();
  const ends = stages
    .map((s) => s.endDate || s.startDate)
    .filter(Boolean)
    .sort();
  const count = plural(stages.length, '{n} stage', '{n} stages');
  const range = dayRange(starts[0] ?? '', ends[ends.length - 1] ?? '');
  return range ? `${count} · ${range}` : count;
}

/** `stages` with the one at `from` moved to `to`. */
export function moveStage(stages: PlanStage[], from: number, to: number): PlanStage[] {
  const moved = stages[from];
  if (!moved || from === to || to < 0 || to >= stages.length) return stages;
  const next = stages.filter((_, i) => i !== from);
  next.splice(to, 0, moved);
  return next;
}
