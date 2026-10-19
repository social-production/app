import type { Proposal, ProposalKind, VoteChoice } from '#lib/domain/governance.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { cellOf } from './projects.ts';

interface ProposalWire {
  id: ActionHash;
  author: AgentPubKey;
  kind: ProposalKind;
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
}

const governance = async <T>(projectId: string, fn: string, payload?: unknown): Promise<T> => {
  const { cell } = await cellOf(projectId);
  return callCell<T>(cell, 'governance', fn, payload);
};

/** Every proposal in a project, newest first. */
export async function listProposals(projectId: string): Promise<Proposal[]> {
  const wire = await governance<ProposalWire[]>(projectId, 'list_proposals');
  return wire.map((p) => ({
    id: hashToString(p.id),
    author: hashToString(p.author),
    kind: p.kind,
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
  }));
}

/** Propose a change; the proposer's Yes comes with it. */
export async function propose(projectId: string, kind: ProposalKind): Promise<void> {
  await governance<ActionHash>(projectId, 'propose', kind);
}

/** Votes are final (ADR 0033). */
export async function castVote(
  projectId: string,
  proposal: string,
  choice: VoteChoice,
): Promise<void> {
  await governance<ActionHash>(projectId, 'vote', { proposal: stringToHash(proposal), choice });
}

/** Write the Outcome of a proposal its votes already decide (anyone may; validators recheck). */
export async function closeProposal(projectId: string, proposal: string): Promise<void> {
  await governance<ActionHash | null>(projectId, 'close', stringToHash(proposal));
}
