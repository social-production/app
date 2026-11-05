import type { Moderation } from './moderation.ts';
import type { NotificationItem } from './notifications.ts';
import { isProjectMode, type CharterMode } from './projects.ts';

type Unread = Omit<NotificationItem, 'isUnread'>;

/**
 * web-backend's `prj-removed` (moderation/effects.py `_end_removed_project`): once a report
 * removes a project, its members and role holders get "This project was removed after a
 * <spam|serious harm> report.", with no actor, opening Home. Projects only, as on web (not
 * events, help requests, threads or posts). The time is the report's: readers replay the
 * votes (ADR 0071) and don't keep when the last one tipped it.
 */
export function removedNotifications(
  p: { seed: string; title: string; mode: CharterMode; involved: boolean },
  moderation: Moderation,
): Unread[] {
  const report = moderation.report;
  if (moderation.state !== 'removed' || !report || !p.involved || !isProjectMode(p.mode)) {
    return [];
  }
  const reason = report.reason === 'spam' ? 'spam' : 'serious harm';
  return [
    {
      id: `prj-removed:${report.id}`,
      kind: 'prj-removed',
      subject: 'project',
      seed: p.seed,
      projectMode: p.mode,
      actor: '',
      title: p.title,
      body: `This project was removed after a ${reason} report.`,
      lead: null,
      target: { home: true },
      createdAt: report.createdAt,
    },
  ];
}
