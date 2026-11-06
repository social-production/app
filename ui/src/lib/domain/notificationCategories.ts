import type { NotificationKind } from './notifications.ts';

/** web's notification settings categories (web-backend `notification_preferences.py`
 * KIND_TO_CATEGORY). */
export type NotificationCategory =
  | 'follows'
  | 'comments'
  | 'shares_invites'
  | 'votes_needed'
  | 'phase_done'
  | 'roles'
  | 'plan_leading';

/** web's `DEFAULT_NOTIFICATION_CATEGORIES` leaves these off until switched on. */
export const OFF_BY_DEFAULT: NotificationCategory[] = ['plan_leading'];

export const categoryOf: Record<NotificationKind, NotificationCategory> = {
  reply: 'comments',
  mention: 'comments',
  'prj-phase-vote': 'votes_needed',
  'prj-phase-done': 'phase_done',
  'new-follower': 'follows',
  'follow-request': 'follows',
  'follow-accepted': 'follows',
  'prj-activity': 'roles',
  'prj-role-suggest': 'roles',
  'prj-role-taken': 'roles',
  'evt-invite': 'shares_invites',
  'community-invite': 'shares_invites',
  'hr-share': 'shares_invites',
  'prj-share': 'shares_invites',
  'evt-share': 'shares_invites',
  'thread-share': 'shares_invites',
  'post-share': 'shares_invites',
  'prj-plan-lead': 'plan_leading',
  'evt-plan-lead': 'plan_leading',
  'prj-removed': 'phase_done',
  'bot-review': 'votes_needed',
  'bot-review-result': 'votes_needed',
  'svc-request': 'roles',
};
