import type { FollowRow, FollowStatus, FollowView } from '#lib/domain/follows.ts';
import {
  callZome,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

const identity = <T>(fn: string, payload?: unknown) =>
  callZome<T>('directory', 'identity', fn, payload);

interface RowWire {
  agent: AgentPubKey;
  status: FollowStatus;
  since: number;
  accepted_at: number | null;
}

interface ViewWire {
  requires_approval: boolean;
  followers: RowWire[];
  following: RowWire[];
  my_status: FollowStatus | null;
}

const toRow = (w: RowWire): FollowRow => ({
  agent: hashToString(w.agent),
  status: w.status,
  since: w.since,
  acceptedAt: w.accepted_at,
});

export async function followView(agent: string): Promise<FollowView> {
  const w = await identity<ViewWire>('follow_view', stringToHash(agent));
  return {
    requiresApproval: w.requires_approval,
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
