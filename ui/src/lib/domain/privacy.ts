import type { FollowView } from './follows.ts';
import type { CharterMode } from './projects.ts';

/**
 * web's Privacy switches (ADR 0112): display rules the device applies. Everything stays
 * public to the network; the app just doesn't show it to the people named.
 */
export type PrivacySwitch = 'hide-profile' | 'hide-posts' | 'hide-from-feeds';

export const PRIVACY_SWITCHES: { name: PrivacySwitch; label: string; note: string }[] = [
  {
    name: 'hide-profile',
    label: 'Hide profile activity from non-followers',
    note: 'Only followers see your projects, threads, events and posts on your profile.',
  },
  {
    name: 'hide-posts',
    label: 'Hide personal posts from non-followers',
    note: 'People who don’t follow you don’t see your posts.',
  },
  {
    name: 'hide-from-feeds',
    label: 'Hide my public activity from others’ personal feeds',
    note: 'Your projects, threads and events stay out of your followers’ Home feeds.',
  },
];

/** Whether `viewer` (null: signed out) counts as an accepted follower of the person `view`
 * describes, or is that person. */
export function insider(view: FollowView, person: string, viewer: string | null): boolean {
  if (!viewer) return false;
  if (viewer === person) return true;
  return view.followers.some((f) => f.agent === viewer && f.status === 'accepted');
}

/** The profile's feed is hidden from `viewer`. */
export const profileHidden = (view: FollowView, person: string, viewer: string | null): boolean =>
  view.privacy.includes('hide-profile') && !insider(view, person, viewer);

/** What Home leaves out (ADR 0112), from the authors' switches. */
export interface FeedRules {
  /** People you follow who keep their public activity out of personal feeds. */
  quietFollows: Set<string>;
  /** Authors whose posts only their followers see. */
  postHiders: Set<string>;
  /** Who you follow (accepted), and you. */
  insiders: Set<string>;
}

/** Whether a card by `creator` of kind `mode` stays on Home's personal rows. */
export function keepPersonal(rules: FeedRules, creator: string, mode: CharterMode): boolean {
  if (mode === 'post') return !rules.postHiders.has(creator) || rules.insiders.has(creator);
  return !rules.quietFollows.has(creator);
}

/** Whether a post by `creator` shows to someone who isn't among `insiders`. */
export const postShown = (rules: FeedRules, creator: string): boolean =>
  !rules.postHiders.has(creator) || rules.insiders.has(creator);

/** Home's rules from who you follow (`insiders` includes you) and the authors found hiding
 * their posts. */
export const feedRules = (
  quiet: string[],
  insiders: string[],
  postHiders: Iterable<string> = [],
): FeedRules => ({
  quietFollows: new Set(quiet),
  postHiders: new Set(postHiders),
  insiders: new Set(insiders),
});

/** The agents among `found` (author → switches) who hide their posts. */
export const postHidersIn = (found: Map<string, string[]>): string[] =>
  [...found].filter(([, p]) => p.includes('hide-posts')).map(([a]) => a);
