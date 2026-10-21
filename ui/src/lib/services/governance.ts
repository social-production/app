import type { Proposal, ProposalKind, VoteChoice } from '#lib/domain/governance.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { cellFor, type ProjectRef } from './projects.ts';

type KindWire =
  | Exclude<ProposalKind, { Phase: unknown }>
  | { Phase: { to: string; plan: ActionHash | null; reason: string } };

interface ProposalWire {
  id: ActionHash;
  author: AgentPubKey;
  kind: KindWire;
  created_at: number;
  yes: number;
  no: number;
  neutral: number;
  population: number;
  required: number;
  status: Proposal['status'];
  closed_at: number | null;
  my_vote: VoteChoice | null;
  settled: boolean;
  added: number;
  missing: number;
  challenge_until: number | null;
  can_challenge: boolean;
}

const governance = async <T>(project: ProjectRef, fn: string, payload?: unknown): Promise<T> =>
  callCell<T>(await cellFor(project), 'governance', fn, payload);

function kindOf(kind: KindWire): ProposalKind {
  if (!('Phase' in kind)) return kind;
  const { to, plan, reason } = kind.Phase;
  return { Phase: { to, plan: plan ? hashToString(plan) : null, reason } };
}

/** Every proposal in a project except plans (they have their own board), newest first. */
export async function listProposals(project: ProjectRef): Promise<Proposal[]> {
  const wire = await governance<ProposalWire[]>(project, 'list_proposals');
  return wire.map((p) => ({
    id: hashToString(p.id),
    author: hashToString(p.author),
    kind: kindOf(p.kind),
    createdAt: p.created_at,
    yes: p.yes,
    no: p.no,
    neutral: p.neutral,
    population: p.population,
    required: p.required,
    status: p.status,
    closedAt: p.closed_at,
    myVote: p.my_vote,
    settled: p.settled,
    added: p.added,
    missing: p.missing,
    challengeUntil: p.challenge_until,
    canChallenge: p.can_challenge,
  }));
}

/** Propose a change; the proposer's Yes comes with it (not for returns, as on web). A phase
 * move's plan is filled in by the zome (the current leading plan). */
export async function propose(projectId: string, kind: ProposalKind): Promise<void> {
  const wire: KindWire =
    'Phase' in kind
      ? { Phase: { to: kind.Phase.to, plan: null, reason: kind.Phase.reason } }
      : kind;
  await governance<ActionHash>(projectId, 'propose', wire);
}

/** Votes are final (ADR 0033). */
export async function castVote(
  projectId: string,
  proposal: string,
  choice: VoteChoice,
): Promise<void> {
  await governance<ActionHash>(projectId, 'vote', { proposal: stringToHash(proposal), choice });
}

/** Count the members a proposal's snapshot left out; it reopens with them (ADR 0040). */
export async function challengeCount(projectId: string, proposal: string): Promise<void> {
  await governance<ActionHash>(projectId, 'challenge', stringToHash(proposal));
}

/** Write the Outcome of a proposal its votes already decide (anyone may; validators recheck). */
export async function closeProposal(projectId: string, proposal: string): Promise<void> {
  await governance<ActionHash | null>(projectId, 'close_proposal', stringToHash(proposal));
}
