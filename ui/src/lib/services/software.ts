import type {
  MergerSource,
  SoftwareGovernance,
  SoftwareRequest,
  SoftwareStage,
  SoftwareTally,
} from '#lib/domain/software.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { cellFor, type ProjectRef } from './projects.ts';

/** Software projects' panel (ADR 0086): the governance zome's `software` read and its proposals. */

interface TallyWire {
  yes: number;
  no: number;
  population: number;
  required: number;
  status: SoftwareTally['status'];
  my_vote: SoftwareTally['myVote'];
  settled: boolean;
  closed_at: number | null;
}

interface Filed {
  id: ActionHash;
  author: AgentPubKey;
  created_at: number;
  tally: TallyWire;
}

interface PullRequestWire extends Filed {
  title: string;
  summary: string;
  pull_request_id: string;
  pull_request_url: string;
  stage: SoftwareStage;
  stage_label: string;
  merge: { proposal: ActionHash; merge_id: string; merge_url: string; by: AgentPubKey } | null;
  can_record_merge: boolean;
}

interface SoftwareWire {
  repository_url: string;
  mergers: { agent: AgentPubKey; source: MergerSource }[];
  candidates: AgentPubKey[];
  pull_requests: PullRequestWire[];
  capability_requests: (Filed & { agent: AgentPubKey; grant: boolean })[];
  repository_requests: (Filed & {
    repository_url: string;
    previous_repository_url: string;
    reason: string;
    pull_request: ActionHash;
  })[];
  can_propose: boolean;
}

const tally = (t: TallyWire): SoftwareTally => ({
  yes: t.yes,
  no: t.no,
  population: t.population,
  required: t.required,
  status: t.status,
  myVote: t.my_vote,
  settled: t.settled,
  closedAt: t.closed_at,
});

const filed = (f: Filed) => ({
  id: hashToString(f.id),
  author: hashToString(f.author),
  createdAt: f.created_at,
  tally: tally(f.tally),
});

export async function loadSoftware(project: ProjectRef): Promise<SoftwareGovernance> {
  const w = await callCell<SoftwareWire>(await cellFor(project), 'governance', 'software');
  return {
    repositoryUrl: w.repository_url,
    mergers: w.mergers.map((m) => ({ agent: hashToString(m.agent), source: m.source })),
    candidates: w.candidates.map(hashToString),
    pullRequests: w.pull_requests.map((p) => ({
      ...filed(p),
      title: p.title,
      summary: p.summary,
      pullRequestId: p.pull_request_id,
      pullRequestUrl: p.pull_request_url,
      stage: p.stage,
      stageLabel: p.stage_label,
      merge: p.merge && {
        proposal: hashToString(p.merge.proposal),
        mergeId: p.merge.merge_id,
        mergeUrl: p.merge.merge_url,
        by: hashToString(p.merge.by),
      },
      canRecordMerge: p.can_record_merge,
    })),
    capabilityRequests: w.capability_requests.map((c) => ({
      ...filed(c),
      agent: hashToString(c.agent),
      grant: c.grant,
    })),
    repositoryRequests: w.repository_requests.map((r) => ({
      ...filed(r),
      repositoryUrl: r.repository_url,
      previousRepositoryUrl: r.previous_repository_url,
      reason: r.reason,
      pullRequest: hashToString(r.pull_request),
    })),
    canPropose: w.can_propose,
  };
}

function wire(request: SoftwareRequest): unknown {
  if ('Merge' in request)
    return { Merge: { ...request.Merge, pull_request: stringToHash(request.Merge.pull_request) } };
  if ('MergeCapability' in request)
    return {
      MergeCapability: {
        ...request.MergeCapability,
        agent: stringToHash(request.MergeCapability.agent),
      },
    };
  if ('Repository' in request)
    return {
      Repository: {
        ...request.Repository,
        pull_request: stringToHash(request.Repository.pull_request),
      },
    };
  return request;
}

/** File a software proposal; members vote on it like any other (no author vote, as on web). */
export async function proposeSoftware(
  project: ProjectRef,
  request: SoftwareRequest,
): Promise<void> {
  await callCell<ActionHash>(await cellFor(project), 'governance', 'propose', {
    Software: wire(request),
  });
}
