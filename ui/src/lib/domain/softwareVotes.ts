/**
 * A software project's votes outside its panel (ADR 0086): the Take part vote sheet (web's
 * `pushSoftwareGovernanceActions`: merge capability and repository votes) and the History tab
 * (web-backend's `build_software_history_entries`). Each becomes a `Software` proposal, so the
 * existing vote sheet and History cards show it and cast its vote.
 */
import type { Proposal, SoftwareVote } from './governance.ts';
import type { HistoryEntry, HistoryStatus } from './history.ts';
import {
  votedProposal,
  type SoftwareGovernance,
  type SoftwarePullRequest,
  type SoftwareStage,
  type SoftwareTally,
} from './software.ts';

type Names = Record<string, string>;

function proposalOf(
  id: string,
  author: string,
  createdAt: number,
  t: SoftwareTally,
  vote: SoftwareVote,
): Proposal {
  return {
    id,
    author,
    kind: { Software: vote },
    createdAt,
    yes: t.yes,
    no: t.no,
    neutral: t.neutral,
    population: t.population,
    required: t.required,
    status: t.status,
    closedAt: t.closedAt,
    myVote: t.myVote,
    settled: t.settled,
    added: t.added,
    missing: t.missing,
    challengeUntil: t.challengeUntil,
    canChallenge: t.canChallenge,
  };
}

const actionLabel = (grant: boolean): string =>
  grant ? 'Grant merge capability' : 'Revoke merge capability';

const open = (t: SoftwareTally): boolean => t.status === 'Open' && !t.settled;

/** web's Take part votes: open merge capability and repository requests. */
export function softwareDockVotes(g: SoftwareGovernance | null, names: Names): Proposal[] {
  if (!g?.canPropose) return [];
  const capability = g.capabilityRequests
    .filter((c) => open(c.tally))
    .map((c) =>
      proposalOf(c.id, c.author, c.createdAt, c.tally, {
        label: 'Merge capability',
        title: actionLabel(c.grant),
        description: '',
        reason: `Member: ${names[c.agent] ?? 'member'}`,
      }),
    );
  const repository = g.repositoryRequests
    .filter((r) => open(r.tally))
    .map((r) =>
      proposalOf(r.id, r.author, r.createdAt, r.tally, {
        label: 'Repository',
        title: r.repositoryUrl,
        description: '',
        reason: r.reason,
      }),
    );
  return [...capability, ...repository];
}

/** web's pull request history status, kept per stage. */
function prStatus(stage: SoftwareStage): HistoryStatus {
  if (stage === 'awaiting-merge' || stage === 'confirmed') return 'approved';
  return stage === 'rejected' || stage === 'replaced' ? 'rejected' : 'open';
}

function statusOf(t: SoftwareTally): HistoryStatus {
  if (t.status === 'Passed') return 'approved';
  return t.status === 'Rejected' ? 'rejected' : 'open';
}

function entry(p: Proposal, kindLabel: string, status: HistoryStatus): HistoryEntry {
  return {
    id: p.id,
    // Only a vote still open is cast from the sheet; the rest show their record.
    proposal: p,
    plan: null,
    kindLabel,
    author: p.author,
    createdAt: p.createdAt,
    status,
    yes: p.yes,
    no: p.no,
    required: p.required,
    population: p.population,
    myVote: p.myVote,
  };
}

function prEntry(pr: SoftwarePullRequest): HistoryEntry {
  const confirming = pr.stage === 'confirmation' || pr.stage === 'confirmed';
  const vote = proposalOf(votedProposal(pr), pr.author, pr.createdAt, pr.tally, {
    label: 'Pull request',
    title: pr.title,
    description: pr.summary,
    reason: pr.pullRequestId,
  });
  return entry(
    vote,
    confirming ? 'Merge confirmation' : 'Pull request approval',
    prStatus(pr.stage),
  );
}

/** web's software history entries: pull requests, merge capability changes, replacements. */
export function softwareHistory(g: SoftwareGovernance | null, names: Names): HistoryEntry[] {
  if (!g) return [];
  const capability = g.capabilityRequests.map((c) => {
    const p = proposalOf(c.id, c.author, c.createdAt, c.tally, {
      label: 'Merge capability',
      title: names[c.agent] ?? 'unknown',
      description: '',
      reason: actionLabel(c.grant),
    });
    return entry(p, 'Merge capability change', statusOf(c.tally));
  });
  const repository = g.repositoryRequests.map((r) => {
    const p = proposalOf(r.id, r.author, r.createdAt, r.tally, {
      label: 'Repository',
      title: r.repositoryUrl,
      description: r.previousRepositoryUrl,
      reason: r.reason,
    });
    return entry(p, 'Repository replacement', statusOf(r.tally));
  });
  return [...g.pullRequests.map(prEntry), ...capability, ...repository];
}
