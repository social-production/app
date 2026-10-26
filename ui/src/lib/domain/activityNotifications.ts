import type { Activity } from './activities.ts';
import type { NotificationItem } from './notifications.ts';
import type { ProjectMode } from './projects.ts';

type Unread = Omit<NotificationItem, 'isUnread'>;

/**
 * web-backend's activity notifications (ADR 0056), derived on this device like the others
 * (ADR 0046): "<activity> was added." to members (`prj-activity`), "You've been suggested for
 * <role> on <activity>." (`prj-role-suggest`) and, to whoever set the activity up, "<name> took
 * the "<role>" role in <activity>." (`prj-role-taken`). web also tells a suggester when their
 * suggestion is declined; the app's view drops declined suggestions, so that one waits.
 */
export function activityNotifications(
  p: { seed: string; title: string; mode: ProjectMode; amMember: boolean },
  activities: Activity[],
  me: string,
  names: Record<string, string>,
): Unread[] {
  const base = { subject: 'project' as const, seed: p.seed, projectMode: p.mode, lead: null };
  const out: Unread[] = [];
  for (const a of activities) {
    const target = { activity: a.id };
    if (p.amMember && a.author !== me) {
      out.push({
        ...base,
        id: `activity:${a.id}`,
        kind: 'prj-activity',
        actor: a.author,
        title: p.title,
        body: `${a.title} was added.`,
        target,
        createdAt: a.createdAt,
      });
    }
    for (const role of a.roles) {
      const s = role.suggestion;
      if (s && s.person === me && s.by !== me) {
        out.push({
          ...base,
          id: `suggest:${s.id}`,
          kind: 'prj-role-suggest',
          actor: s.by,
          title: `Suggested for ${role.label}`,
          body: `You've been suggested for ${role.label} on ${a.title}.`,
          target,
          createdAt: s.at,
        });
      }
      if (a.author !== me) continue;
      role.people.forEach((agent, i) => {
        if (agent === me) return;
        out.push({
          ...base,
          id: `taken:${a.id}:${role.index}:${agent}`,
          kind: 'prj-role-taken',
          actor: agent,
          title: a.title,
          body: `${names[agent] ?? 'Someone'} took the "${role.label}" role in ${a.title}.`,
          target,
          createdAt: role.takenAt[i] ?? a.createdAt,
        });
      });
    }
  }
  return out;
}
