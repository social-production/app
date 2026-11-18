import type { FollowRow, FollowStatus, FollowView } from '#lib/domain/follows.ts';
import type { PrivacySwitch } from '#lib/domain/privacy.ts';
import {
  callZome,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';
import { myAgent } from './people.ts';

const identity = <T>(fn: string, payload?: unknown) =>
  callZome<T>('directory', 'identity', fn, payload);

interface RowWire {
  agent: AgentPubKey;
  status: FollowStatus;
  since: number;
  accepted_at: number | null;
  privacy?: PrivacySwitch[];
}

interface ViewWire {
  requires_approval: boolean;
  privacy?: PrivacySwitch[];
  followers: RowWire[];
  following: RowWire[];
  my_status: FollowStatus | null;
}

const toRow = (w: RowWire): FollowRow => ({
  agent: hashToString(w.agent),
  status: w.status,
  since: w.since,
  acceptedAt: w.accepted_at,
  privacy: w.privacy ?? [],
});

export async function followView(agent: string): Promise<FollowView> {
  const w = await identity<ViewWire>('follow_view', stringToHash(agent));
  return {
    requiresApproval: w.requires_approval,
    privacy: w.privacy ?? [],
    followers: w.followers.map(toRow),
    following: w.following.map(toRow),
    myStatus: w.my_status,
  };
}

/** Follow someone; pending while they require approval. */
export async function follow(agent: string): Promise<FollowStatus | null> {
  return identity<FollowStatus | null>('follow', stringToHash(agent));
}

export async function unfollow(agent: string): Promise<void> {
  await identity<null>('unfollow', stringToHash(agent));
}

export async function decideFollow(follower: string, accept: boolean): Promise<void> {
  await identity<ActionHash>(accept ? 'accept_follow' : 'reject_follow', stringToHash(follower));
}

export async function followRequests(): Promise<FollowRow[]> {
  return (await identity<RowWire[]>('follow_requests')).map(toRow);
}

export async function setFollowApproval(on: boolean): Promise<void> {
  await identity<boolean>('set_follow_approval', on);
}

/** Turn one of your privacy switches on or off (ADR 0112); returns your switches after. */
export async function setPrivacy(name: PrivacySwitch, on: boolean): Promise<PrivacySwitch[]> {
  return identity<PrivacySwitch[]>('set_privacy', { name, on });
}

/** The switches of `agents` (authors in a list); people with none are left out. */
export async function privacyOf(agents: string[]): Promise<Map<string, PrivacySwitch[]>> {
  const unique = [...new Set(agents)];
  if (!unique.length) return new Map();
  try {
    const rows = await identity<{ agent: AgentPubKey; privacy: PrivacySwitch[] }[]>(
      'privacy_of',
      unique.map(stringToHash),
    );
    return new Map(rows.map((r) => [hashToString(r.agent), r.privacy]));
  } catch {
    return new Map(); // Best effort: an unreadable switch shows the post.
  }
}

/**
 * The accepted follows around you, for ranking @mention suggestions as web-backend does
 * (people you follow, then your followers). Empty when the directory can't be read.
 */
export async function followCircles(): Promise<{
  following: string[];
  followers: string[];
  /** People you follow who keep their activity out of personal feeds (ADR 0112). */
  quiet: string[];
}> {
  try {
    const view = await followView(await myAgent());
    const accepted = (rows: FollowRow[]) => rows.filter((r) => r.status === 'accepted');
    const agents = (rows: FollowRow[]) => accepted(rows).map((r) => r.agent);
    return {
      following: agents(view.following),
      followers: agents(view.followers),
      quiet: accepted(view.following)
        .filter((r) => r.privacy.includes('hide-from-feeds'))
        .map((r) => r.agent),
    };
  } catch {
    return { following: [], followers: [], quiet: [] };
  }
}
