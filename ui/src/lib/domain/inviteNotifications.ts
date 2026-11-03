import type { NotificationItem } from './notifications.ts';

type Unread = Omit<NotificationItem, 'isUnread'>;

/** What a private invite is to. */
export type InviteKind = 'event' | 'community';

/** web-backend's `evt-invite` / `community-invite`, one per invite still waiting for an
 * answer (ADR 0061, 0068). */
export function inviteNotifications(
  invites: { seed: string; title: string; from: string; at: number; kind: InviteKind }[],
): Unread[] {
  return invites.map((i) => ({
    id: `evt-invite:${i.seed}`,
    kind: i.kind === 'community' ? ('community-invite' as const) : ('evt-invite' as const),
    subject: 'invites' as const,
    seed: i.seed,
    projectMode: i.kind,
    actor: i.from,
    title: i.title,
    body:
      i.kind === 'community'
        ? `You were invited to join ${i.title}. Accept or decline it in Invites.`
        : `You were invited to an event: ${i.title}. Accept or decline it in Invites.`,
    lead: null,
    target: null,
    createdAt: i.at,
  }));
}
