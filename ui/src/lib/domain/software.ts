/** Software projects (ADR 0086): what web's software panel shows, in the app's shape. */

export type SoftwareStage =
  'approval' | 'awaiting-merge' | 'confirmation' | 'confirmed' | 'rejected' | 'replaced';

export type MergerSource = 'plan-creator' | 'approved-request' | 'repo-replacement';

export interface SoftwareTally {
  yes: number;
  no: number;
  population: number;
  required: number;
  status: 'Open' | 'Passed' | 'Rejected';
  myVote: 'Yes' | 'No' | 'Neutral' | null;
  settled: boolean;
  closedAt: number | null;
}

export interface SoftwareMerge {
  proposal: string;
  mergeId: string;
  mergeUrl: string;
  by: string;
}

export interface SoftwarePullRequest {
  id: string;
  author: string;
  createdAt: number;
  title: string;
  summary: string;
  pullRequestId: string;
  pullRequestUrl: string;
  stage: SoftwareStage;
  stageLabel: string;
  merge: SoftwareMerge | null;
  tally: SoftwareTally;
  canRecordMerge: boolean;
}

export interface SoftwareCapabilityRequest {
  id: string;
  author: string;
  createdAt: number;
  agent: string;
  grant: boolean;
  tally: SoftwareTally;
}

export interface SoftwareRepositoryRequest {
  id: string;
  author: string;
  createdAt: number;
  repositoryUrl: string;
  previousRepositoryUrl: string;
  reason: string;
  pullRequest: string;
  tally: SoftwareTally;
}

export interface SoftwareGovernance {
  repositoryUrl: string;
  mergers: { agent: string; source: MergerSource }[];
  candidates: string[];
  pullRequests: SoftwarePullRequest[];
  capabilityRequests: SoftwareCapabilityRequest[];
  repositoryRequests: SoftwareRepositoryRequest[];
  canPropose: boolean;
}

export type SoftwareRequest =
  | {
      PullRequest: {
        title: string;
        summary: string;
        pull_request_id: string;
        pull_request_url: string;
      };
    }
  | { Merge: { pull_request: string; merge_id: string; merge_url: string } }
  | { MergeCapability: { agent: string; grant: boolean } }
  | { Repository: { repository_url: string; reason: string; pull_request: string } };

/** web's `isInProgressPullRequest`. */
export function inProgress(pr: SoftwarePullRequest): boolean {
  return pr.stage === 'approval' || pr.stage === 'awaiting-merge' || pr.stage === 'confirmation';
}

/** The vote a member may cast now: web's `viewerCanVote && canStillPass && !passes`. */
export function canVote(pr: SoftwarePullRequest): boolean {
  const open = pr.stage === 'approval' || pr.stage === 'confirmation';
  return open && pr.tally.status === 'Open' && !pr.tally.settled && pr.tally.myVote === null;
}

/** web's stage dots: how many of the three stages are reached. */
export function stageFilled(stage: SoftwareStage): number {
  if (stage === 'confirmation' || stage === 'confirmed') return 3;
  return stage === 'awaiting-merge' ? 2 : 1;
}

export function stageTooltip(stage: SoftwareStage): string {
  if (stage === 'awaiting-merge') return 'Stage 2 of 3: merge needed';
  if (stage === 'confirmation') return 'Stage 3 of 3: merge confirmation needed';
  return 'Stage 1 of 3: pull request vote needed';
}

export function stageHelperCopy(stage: SoftwareStage): string {
  if (stage === 'awaiting-merge')
    return 'Awaiting a merge-capable member to record the merge ID and merge link.';
  if (stage === 'confirmation')
    return 'Awaiting confirmation that the recorded merge matches this pull request.';
  return 'Awaiting approval before anyone can record a merge.';
}

/** web's `normalizeExternalUrl`. */
export function externalUrl(value: string | null | undefined): string {
  const trimmed = value?.trim() ?? '';
  if (!trimmed) return '';
  if (/^[a-zA-Z][a-zA-Z\d+.-]*:/.test(trimmed)) return trimmed;
  return `https://${trimmed}`;
}

const APPROVAL_PERCENT = 66;

function approval(t: SoftwareTally): number {
  const voted = t.yes + t.no;
  return voted > 0 ? Math.round((t.yes / voted) * 100) : 0;
}

/** web's `formatCompactVoteStatus(summary, 66)`. */
export function compactVoteStatus(t: SoftwareTally): string {
  return `${approval(t)}% yes · ${t.yes + t.no}/${t.population} voted · ${APPROVAL_PERCENT}% needed`;
}

/** web's `formatProjectVoteSummary`. */
export function voteSummaryLine(t: SoftwareTally): string {
  return `${approval(t)}% approval · ${t.yes} yes · ${t.no} no`;
}

/** web's `formatProjectVoteRequirement` (the fewest more Yes votes that pass it). */
export function voteRequirementLine(t: SoftwareTally): string {
  const total = t.yes + t.no;
  const left = Math.max(t.population - total, 0);
  let needed = 0;
  if (t.required > 0) {
    for (let more = 0; more <= left; more++) {
      const finalTotal = total + more + Math.max(t.required - total - more, 0);
      if (finalTotal - total > left) continue;
      if (finalTotal > 0 && (t.yes + more) * 100 >= APPROVAL_PERCENT * finalTotal) {
        needed = more;
        break;
      }
    }
  }
  const need = needed > 0 ? `Needs ${needed} more yes` : 'Enough yes votes so far';
  const waiting =
    left > 0 ? ` · ${left} ${left === 1 ? 'person has' : 'people have'} not voted` : '';
  return `${need} (${APPROVAL_PERCENT}% approval)${waiting}`;
}
