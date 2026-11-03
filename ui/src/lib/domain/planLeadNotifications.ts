import type { NotificationItem } from './notifications.ts';
import type { PlanKind } from './plans.ts';
import type { CharterMode } from './projects.ts';

type Unread = Omit<NotificationItem, 'isUnread'>;

/** The governance zome's `LeadChange`: a plan took its kind's lead when `by` closed a plan
 * vote at `at` (ADR 0072). */
export interface LeadChange {
  plan: string;
  author: string;
  kind: PlanKind;
  at: number;
  by: string;
}

/**
 * web-backend's `prj-plan-lead` / `evt-plan-lead` (projects_plans.py, events_plans.py): to the
 * new leader's author, "A vote passed and your plan is now leading.", opening the page.
 */
export function planLeadNotifications(
  p: { seed: string; mode: CharterMode },
  changes: LeadChange[],
  me: string,
): Unread[] {
  const event = p.mode === 'event';
  return changes
    .filter((c) => c.author === me)
    .map((c) => ({
      id: `plan-lead:${c.plan}:${c.at}`,
      kind: event ? ('evt-plan-lead' as const) : ('prj-plan-lead' as const),
      subject: 'project' as const,
      seed: p.seed,
      projectMode: p.mode,
      actor: c.by,
      title: 'Plan became leading',
      body: event
        ? 'A vote passed and your event plan is now leading.'
        : 'A vote passed and your plan is now leading.',
      lead: null,
      target: null,
      createdAt: c.at,
    }));
}
