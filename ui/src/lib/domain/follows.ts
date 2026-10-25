/** Mirrors the identity zome's follows (ADR 0054). Rejected follows read as no follow. */

export type FollowStatus = 'accepted' | 'pending';

export interface FollowRow {
  agent: string;
  status: FollowStatus;
  /** When they followed (Holochain micros). */
  since: number;
  /** When an approval-gated follow was accepted (micros). */
  acceptedAt: number | null;
}

export interface FollowView {
  requiresApproval: boolean;
  followers: FollowRow[];
  following: FollowRow[];
  myStatus: FollowStatus | null;
}

/** As on web, counts and others' lists show accepted follows only; your own shows pending too. */
export function shownRows(rows: FollowRow[], own: boolean): FollowRow[] {
  return own ? rows : rows.filter((r) => r.status === 'accepted');
}

export function acceptedCount(rows: FollowRow[]): number {
  return rows.filter((r) => r.status === 'accepted').length;
}
