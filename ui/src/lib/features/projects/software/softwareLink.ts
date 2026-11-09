import type { SoftwareState } from './softwareState.svelte.ts';

/**
 * web's software vote links (`?open=vote&voteKind=pull_request&voteTarget=…&assess=1` from the
 * rail, `voteKind=pull_request_merge` from the rail and the `pr-approved` notification): once
 * the pull requests load, open its wizard (vote or record merge) when the viewer can act, else
 * scroll to its card. Each link opens once.
 */
export function openSoftwareLink(sw: SoftwareState, params: Pick<URLSearchParams, 'get'>): void {
  const kind = params.get('voteKind');
  const target = params.get('voteTarget');
  if (params.get('open') !== 'vote' || !target || !sw.governance) return;
  if (kind !== 'pull_request' && kind !== 'pull_request_merge') return;
  if (sw.linked === target) return;
  const pr = sw.governance.pullRequests.find((p) => p.id === target);
  if (!pr) return;
  sw.linked = target;
  const voting = pr.stage === 'approval' || pr.stage === 'confirmation';
  if (kind === 'pull_request_merge' && pr.canRecordMerge) sw.open('record-merge', pr);
  else if (kind === 'pull_request' && params.get('assess') === '1' && voting)
    sw.open('vote-pr', pr);
  else
    setTimeout(() => {
      document.getElementById(`software-pr-card-${pr.id}`)?.scrollIntoView({ block: 'center' });
    }, 180);
}
