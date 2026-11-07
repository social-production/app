/** Mirrors the project DNA's governance zome (Proposal → Vote → Outcome, ADR 0033). */

import type { Place } from './places.ts';

export type VoteChoice = 'Yes' | 'No' | 'Neutral';
export type ProposalStatus = 'Open' | 'Passed' | 'Rejected';

export type ProposalKind =
  | {
      Edit: {
        title: string | null;
        description: string | null;
        location: string | null;
        /** With `location`: its place (ADR 0078); none leaves the label without one. */
        place?: Place | null;
      };
    }
  | { Update: { body: string } }
  /** `plan`: the approved plan a move out of a plan phase goes ahead with (ADR 0039);
   * `reason`: web's required "why" for a phase change. */
  | { Phase: { to: string; plan: string | null; reason: string } }
  /** web's revert request: back to an earlier planning phase, with a reason (ADR 0040). */
  | { Return: { to: string; reason: string } }
  /** A channel's or community's member vote to remove someone (ADR 0068). */
  | { RemoveMember: { agent: string } }
  /** A service's request settings (ADR 0076): voted in a collective service, saved at once
   * by a personal service's creator. */
  | {
      RequestSettings: {
        enabled: boolean;
        mode: 'calendar' | 'direct' | 'both';
        off_schedule: boolean;
        reason: string;
      };
    };

export interface Proposal {
  id: string;
  author: string;
  kind: ProposalKind;
  createdAt: number;
  yes: number;
  no: number;
  neutral: number;
  population: number;
  required: number;
  status: ProposalStatus;
  closedAt: number | null;
  myVote: VoteChoice | null;
  /** Decided by its votes, but not closed yet; the app closes it on the next load. */
  settled: boolean;
  /** Members a challenge added to the count (ADR 0040). */
  added: number;
  /** Members from before the proposal its count still leaves out. */
  missing: number;
  /** Challenges count until then (48 h after it first closed); null while undecided. */
  challengeUntil: number | null;
  canChallenge: boolean;
}

/** An update that passed (or, for personal services, was posted). */
export interface ProjectUpdate {
  proposal: string;
  author: string;
  body: string;
  at: number;
}

/** Needs 66% yes, as on web (sp_governance::Threshold::DEFAULT). */
export const APPROVAL_PERCENT = 66;

export function kindName(
  kind: ProposalKind,
): 'edit' | 'update' | 'phase' | 'return' | 'remove' | 'settings' {
  if ('Edit' in kind) return 'edit';
  if ('RequestSettings' in kind) return 'settings';
  if ('RemoveMember' in kind) return 'remove';
  if ('Return' in kind) return 'return';
  return 'Update' in kind ? 'update' : 'phase';
}

/** A move or a return: web allows one open phase vote at a time. */
export function isPhaseChange(
  kind: ProposalKind,
): kind is Extract<ProposalKind, { Phase: unknown } | { Return: unknown }> {
  return 'Phase' in kind || 'Return' in kind;
}

/** The sheet title for a proposal (web's pending-vote labels). */
export function proposalLabel(kind: ProposalKind, phaseLabel: (id: string) => string): string {
  if ('Edit' in kind) return 'Edit project';
  if ('Update' in kind) return 'Post update';
  if ('RemoveMember' in kind) return 'Remove member';
  if ('RequestSettings' in kind) return 'Request settings';
  // web's phaseChangeDecisionTitle('return', …).
  if ('Return' in kind) return `Return to ${phaseLabel(kind.Return.to)}`;
  return `Move to ${phaseLabel(kind.Phase.to)}`;
}

/** web's VoteDecisionBody outcome sentence. */
export function outcomeSentence(
  kind: ProposalKind,
  from: string,
  to: (id: string) => string,
): string {
  if ('Update' in kind) return 'If this passes, this update is posted.';
  if ('RemoveMember' in kind) return 'If this passes, this member is removed.';
  if ('RequestSettings' in kind) return 'If this passes, the request settings change.';
  if ('Phase' in kind) return `If this passes, this moves from ${from} to ${to(kind.Phase.to)}.`;
  if ('Return' in kind) return `If this passes, this moves from ${from} to ${to(kind.Return.to)}.`;
  const e = kind.Edit;
  const names = [
    e.title !== null && 'title',
    e.description !== null && 'description',
    e.location !== null && 'location',
  ].filter((n): n is string => !!n);
  const list =
    names.length > 1
      ? `${names.slice(0, -1).join(', ')} and ${names.at(-1) ?? ''}`
      : (names[0] ?? 'details');
  return `If this passes, the ${list} ${names.length > 1 ? 'change' : 'changes'}. Everything else stays as it is.`;
}

/** "2 yes · 1 no · 67%", or web's "No one has voted yet". */
export function tallyLine(p: Pick<Proposal, 'yes' | 'no'>): string {
  const total = p.yes + p.no;
  if (total === 0) return 'No one has voted yet';
  return `${p.yes} yes · ${p.no} no · ${Math.round((p.yes / total) * 100)}%`;
}
