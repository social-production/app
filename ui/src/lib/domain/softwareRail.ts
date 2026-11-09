/**
 * A software project's right rail rows (ADR 0086), as web-backend's `bootstrap/activity_rail.py`
 * and `activity_rail_history.py` build them from its pull request, merge capability and
 * repository replacement tables.
 */
import { voteQuery, type RailItem, type RailProject, type RailVoteKind } from './activityRail.ts';
import type { VoteChoice } from './governance.ts';
import type { SoftwareStage, SoftwareTally } from './software.ts';

/** web-backend's per-kind limit. */
const VOTE_LIMIT = 3;

const tally = (t: { yes: number; no: number }): string => `${t.yes} yes / ${t.no} no`;
const toCast = (t: SoftwareTally): boolean => t.status === 'Open' && !t.settled && !t.myVote;
const grantLabel = (grant: boolean): string => (grant ? 'Grant' : 'Revoke');

function newest<T extends { createdAt: number }>(rows: T[]): T[] {
  return rows
    .slice()
    .sort((a, b) => b.createdAt - a.createdAt)
    .slice(0, VOTE_LIMIT);
}

function row(
  p: RailProject,
  id: string,
  kind: RailVoteKind,
  query: string,
  meta: string,
  createdAt: number,
  countLabel: string,
): RailItem {
  return {
    id,
    seed: p.seed,
    kind: 'vote',
    title: p.title,
    query,
    meta,
    createdAt,
    countLabel,
    projectMode: p.mode,
    voteKindLabel: kind,
  };
}

type Software = NonNullable<RailProject['software']>;

function each<T>(projects: RailProject[], pick: (p: RailProject, s: Software) => T[]): T[] {
  return projects.flatMap((p) => (p.software ? pick(p, p.software) : []));
}

function pullRequestVotes(projects: RailProject[]): RailItem[] {
  const open = each(projects, (p, s) =>
    s.governance.pullRequests
      .filter((r) => (r.stage === 'approval' || r.stage === 'confirmation') && toCast(r.tally))
      .map((r) => ({ p, r, createdAt: r.createdAt })),
  );
  return newest(open).map(({ p, r }) =>
    row(
      p,
      r.id,
      'pull_request',
      voteQuery('pull_request', r.id, true),
      `${r.stage === 'confirmation' ? 'Merge confirmation' : 'Pull request vote'} · ${r.title}`,
      r.createdAt,
      tally(r.tally),
    ),
  );
}

/** web's "Action · merge needed" for members who can merge. */
function mergesNeeded(projects: RailProject[]): RailItem[] {
  const waiting = each(projects, (p, s) =>
    s.governance.pullRequests
      .filter((r) => r.stage === 'awaiting-merge' && !r.merge && r.canRecordMerge)
      .map((r) => ({ p, r, createdAt: r.createdAt })),
  );
  return newest(waiting).map(({ p, r }) =>
    row(
      p,
      `merge:${r.id}`,
      'pull_request_merge',
      voteQuery('pull_request_merge', r.id),
      `Merge needed · ${r.title}`,
      r.createdAt,
      'Record merge',
    ),
  );
}

function capabilityVotes(projects: RailProject[]): RailItem[] {
  const open = each(projects, (p, s) =>
    s.governance.capabilityRequests
      .filter((c) => toCast(c.tally))
      .map((c) => ({ p, c, name: s.names[c.agent], createdAt: c.createdAt })),
  );
  return newest(open).map(({ p, c, name }) =>
    row(
      p,
      c.id,
      'merge_capability',
      voteQuery('merge_capability', c.id),
      `Merge capability · ${grantLabel(c.grant)} ${name ?? 'member'}`,
      c.createdAt,
      tally(c.tally),
    ),
  );
}

function repositoryVotes(projects: RailProject[]): RailItem[] {
  const open = each(projects, (p, s) =>
    s.governance.repositoryRequests
      .filter((r) => toCast(r.tally))
      .map((r) => ({ p, r, createdAt: r.createdAt })),
  );
  return newest(open).map(({ p, r }) =>
    row(
      p,
      r.id,
      'repository_replacement',
      voteQuery('repository_replacement', r.id),
      `Repository replacement · ${r.repositoryUrl}`,
      r.createdAt,
      tally(r.tally),
    ),
  );
}

/** Votes to cast: pull requests (approval, confirmation), merges to record, capability, repo. */
export function softwareVotes(projects: RailProject[]): RailItem[] {
  return [
    ...pullRequestVotes(projects),
    ...mergesNeeded(projects),
    ...capabilityVotes(projects),
    ...repositoryVotes(projects),
  ];
}

/** A vote you cast, as `activityRailHistory.ts` words it. */
export interface SoftwareCast {
  id: string;
  p: RailProject;
  kind: RailVoteKind;
  query: string;
  title: string;
  meta: string;
  createdAt: number;
  vote: VoteChoice;
  outcome: 'passed' | 'failed' | 'open';
}

/**
 * web's `_vote_outcome` of a pull request's stage. web leaves a confirmed merge "Still open"
 * (its stage isn't in the passed set); the app counts it as passed (PARITY: owner review).
 */
function stageOutcome(stage: SoftwareStage): SoftwareCast['outcome'] {
  if (stage === 'awaiting-merge' || stage === 'confirmed') return 'passed';
  return stage === 'rejected' ? 'failed' : 'open';
}

function tallyOutcome(t: SoftwareTally): SoftwareCast['outcome'] {
  if (t.status === 'Passed') return 'passed';
  return t.status === 'Rejected' ? 'failed' : 'open';
}

interface Voted {
  prefix: string;
  target: string;
  kind: RailVoteKind;
  tally: SoftwareTally;
  createdAt: number;
  title: string;
  meta: string;
  outcome: SoftwareCast['outcome'];
}

function castOf(p: RailProject, v: Voted): SoftwareCast[] {
  if (!v.tally.myVote) return [];
  return [
    {
      id: `${v.prefix}${v.target}`,
      p,
      kind: v.kind,
      query: voteQuery(v.kind, v.target),
      title: v.title,
      meta: v.meta,
      createdAt: v.createdAt,
      vote: v.tally.myVote,
      outcome: v.outcome,
    },
  ];
}

/** History: "You voted Yes · <project>" on a pull request; capability and repository votes. */
export function softwareCasts(projects: RailProject[]): SoftwareCast[] {
  return projects.flatMap((p) => {
    const g = p.software?.governance;
    if (!g) return [];
    const voted: Voted[] = [
      ...g.pullRequests.map((r) => ({
        prefix: 'vote-project-pr-',
        target: r.id,
        kind: 'pull_request' as const,
        tally: r.tally,
        createdAt: r.createdAt,
        title: r.title,
        meta: p.title,
        outcome: stageOutcome(r.stage),
      })),
      ...g.capabilityRequests.map((c) => ({
        prefix: 'vote-project-merge-capability-',
        target: c.id,
        kind: 'merge_capability' as const,
        tally: c.tally,
        createdAt: c.createdAt,
        title: `Merge capability · ${p.title}`,
        meta: 'Merge capability',
        outcome: tallyOutcome(c.tally),
      })),
      ...g.repositoryRequests.map((r) => ({
        prefix: 'vote-project-repository-',
        target: r.id,
        kind: 'repository_replacement' as const,
        tally: r.tally,
        createdAt: r.createdAt,
        title: `Repository replacement · ${p.title}`,
        meta: 'Repository replacement',
        outcome: tallyOutcome(r.tally),
      })),
    ];
    return voted.flatMap((v) => castOf(p, v));
  });
}
