import type { SoftwarePullRequest, SoftwareRequest } from '#lib/domain/software.ts';

import type { SoftwareMode } from './softwareState.svelte.ts';

/** web's SoftwareGovernanceWizard: its forms, titles, steps, gates and validation copy. */
export interface SoftwareForms {
  title: string;
  summary: string;
  pullRequestId: string;
  pullRequestUrl: string;
  mergeId: string;
  mergeUrl: string;
  targetAgent: string;
  action: 'grant' | 'revoke';
  repositoryUrl: string;
  reason: string;
  relatedPullRequest: string;
}

export function emptyForms(candidate = '', replaceable = ''): SoftwareForms {
  return {
    title: '',
    summary: '',
    pullRequestId: '',
    pullRequestUrl: '',
    mergeId: '',
    mergeUrl: '',
    targetAgent: candidate,
    action: 'grant',
    repositoryUrl: '',
    reason: '',
    relatedPullRequest: replaceable,
  };
}

export function wizardTitle(mode: SoftwareMode | null, pr: SoftwarePullRequest | null): string {
  switch (mode) {
    case 'create-pr':
      return 'Submit pull request';
    case 'vote-pr':
      return pr?.stage === 'confirmation'
        ? 'Merge confirmation needed'
        : 'Pull request vote needed';
    case 'record-merge':
      return 'Merge needed';
    case 'merge-capability':
      return 'Merge capability change';
    case 'repository-replacement':
      return 'Replace repository';
    default:
      return 'Software governance';
  }
}

export function stepCount(mode: SoftwareMode | null): number {
  if (mode === 'create-pr' || mode === 'repository-replacement') return 4;
  return mode === 'merge-capability' ? 3 : 1;
}

export function submitLabel(mode: SoftwareMode | null): string {
  switch (mode) {
    case 'create-pr':
      return 'Submit for approval';
    case 'vote-pr':
      return 'Done';
    case 'record-merge':
      return 'Submit merge details';
    case 'merge-capability':
      return 'Submit vote request';
    case 'repository-replacement':
      return 'Submit replacement vote';
    default:
      return 'Finish';
  }
}

const filled = (value: string): boolean => !!value.trim();

/** web's `validationMessage`; `null` when the step may go on. */
export function validation(
  mode: SoftwareMode | null,
  step: number,
  f: SoftwareForms,
): string | null {
  switch (mode) {
    case 'create-pr':
      if (step === 0 && !filled(f.title)) return 'Add a title to continue.';
      if (step === 1 && (!filled(f.pullRequestId) || !filled(f.pullRequestUrl)))
        return 'Add both the PR ID and the pull request URL to continue.';
      if (step === 2 && !filled(f.summary)) return 'Add a short summary to continue.';
      return null;
    case 'record-merge':
      if (!filled(f.mergeId) || !filled(f.mergeUrl))
        return 'Enter both the merge ID and the merge link to continue.';
      return null;
    case 'merge-capability':
      if (step === 0 && !filled(f.targetAgent)) return 'Choose a project member to continue.';
      return null;
    case 'repository-replacement':
      if (step === 0 && !filled(f.relatedPullRequest))
        return 'Choose the blocked pull request that triggered this change.';
      if (step === 1 && !filled(f.repositoryUrl))
        return 'Add the replacement repository URL to continue.';
      if (step === 2 && !filled(f.reason)) return 'Explain why the repository must change.';
      return null;
    default:
      return null;
  }
}

/** What the last step files, or `null` (voting files nothing). */
export function request(
  mode: SoftwareMode | null,
  f: SoftwareForms,
  pr: SoftwarePullRequest | null,
): SoftwareRequest | null {
  switch (mode) {
    case 'create-pr':
      return {
        PullRequest: {
          title: f.title.trim(),
          summary: f.summary.trim(),
          pull_request_id: f.pullRequestId.trim(),
          pull_request_url: f.pullRequestUrl.trim(),
        },
      };
    case 'record-merge':
      return pr
        ? {
            Merge: {
              pull_request: pr.id,
              merge_id: f.mergeId.trim(),
              merge_url: f.mergeUrl.trim(),
            },
          }
        : null;
    case 'merge-capability':
      return { MergeCapability: { agent: f.targetAgent, grant: f.action === 'grant' } };
    case 'repository-replacement':
      return {
        Repository: {
          repository_url: f.repositoryUrl.trim(),
          reason: f.reason.trim(),
          pull_request: f.relatedPullRequest,
        },
      };
    default:
      return null;
  }
}
