import type { NotificationItem } from './notifications.ts';
import type { CharterMode } from './projects.ts';
import type { SoftwareGovernance } from './software.ts';

type Unread = Omit<NotificationItem, 'isUnread'>;

/**
 * web-backend's `pr-approved` (software/pull_requests.py, ADR 0086): to a pull request's
 * author once its vote passes, "Voting passed and your pull request is approved for merge.",
 * opening the merge (`?open=vote&voteKind=pull_request_merge`).
 */
export function prApprovedNotifications(
  p: { seed: string; mode: CharterMode },
  g: SoftwareGovernance,
  me: string,
): Unread[] {
  return g.pullRequests.flatMap((pr) =>
    pr.author === me && pr.approved
      ? [
          {
            id: `pr-approved:${pr.id}`,
            kind: 'pr-approved' as const,
            subject: 'project' as const,
            seed: p.seed,
            projectMode: p.mode,
            actor: pr.approved.by,
            title: 'Pull request approved',
            body: 'Voting passed and your pull request is approved for merge.',
            lead: null,
            target: {
              open: 'vote' as const,
              voteTarget: pr.id,
              voteKind: 'pull_request_merge' as const,
            },
            createdAt: pr.approved.at,
          },
        ]
      : [],
  );
}
