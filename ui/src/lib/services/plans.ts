import type { ProposalStatus, VoteChoice } from '#lib/domain/governance.ts';
import type { Plan, PlanBoard, PlanKind, PlanRating } from '#lib/domain/plans.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { cellFor, type ProjectRef } from './projects.ts';

interface PlanViewWire {
  id: ActionHash;
  author: AgentPubKey;
  created_at: number;
  plan: Plan;
  tally: {
    yes: number;
    no: number;
    neutral: number;
    population: number;
    required: number;
    status: ProposalStatus;
    my_vote: VoteChoice | null;
    settled: boolean;
  };
  approved: boolean;
  leader: 'Leading' | 'Tied' | null;
  criteria: {
    id: string;
    label: string;
    kind: 'Rubric' | 'Value';
    mine: PlanRating | null;
    count: number;
    average_x100: number;
  }[];
  average_x100: number | null;
}

interface PlanBoardWire {
  plans: PlanViewWire[];
  open_kind: PlanKind | null;
  gate: PlanKind | null;
  gate_leader: ActionHash | null;
  members: number;
  required: number;
}

const governance = async <T>(project: ProjectRef, fn: string, payload?: unknown): Promise<T> =>
  callCell<T>(await cellFor(project), 'governance', fn, payload);

/** Every plan with its assessment, and what leaving the current phase needs. */
export async function listPlans(project: ProjectRef): Promise<PlanBoard> {
  const wire = await governance<PlanBoardWire>(project, 'list_plans');
  return {
    plans: wire.plans.map((p) => ({
      id: hashToString(p.id),
      author: hashToString(p.author),
      createdAt: p.created_at,
      plan: p.plan,
      tally: {
        yes: p.tally.yes,
        no: p.tally.no,
        neutral: p.tally.neutral,
        population: p.tally.population,
        required: p.tally.required,
        status: p.tally.status,
        myVote: p.tally.my_vote,
        settled: p.tally.settled,
      },
      approved: p.approved,
      leader: p.leader,
      criteria: p.criteria.map((c) => ({
        id: c.id,
        label: c.label,
        kind: c.kind,
        mine: c.mine,
        count: c.count,
        average: c.average_x100 / 100,
      })),
      average: p.average_x100 === null ? null : p.average_x100 / 100,
    })),
    openKind: wire.open_kind,
    gate: wire.gate,
    gateLeader: wire.gate_leader ? hashToString(wire.gate_leader) : null,
    members: wire.members,
    required: wire.required,
  };
}

/** A plan is a proposal; its author assesses and votes like everyone else. */
export async function submitPlan(projectId: string, plan: Plan): Promise<void> {
  await governance<ActionHash>(projectId, 'propose', { Plan: plan });
}

/** Rate one criterion (1–5), or clear it with `null`. The latest rating counts. */
export async function ratePlan(
  projectId: string,
  plan: string,
  criterion: string,
  rating: PlanRating | null,
): Promise<void> {
  await governance<ActionHash>(projectId, 'rate', {
    proposal: stringToHash(plan),
    criterion,
    rating,
  });
}
