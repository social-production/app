/** What a decided vote did, in the past tense (owner, 2026-11-18: History's sheets said "If
 * this passes" about votes long closed). */
import { outcomeSentence, type Proposal, type ProposalKind } from './governance.ts';

function passedSentence(kind: ProposalKind, from: string, to: (id: string) => string): string {
  if ('Update' in kind) return 'This update was posted.';
  if ('RemoveMember' in kind) return 'This member was removed.';
  if ('RequestSettings' in kind) return 'The request settings changed.';
  if ('Software' in kind) return 'This change was applied.';
  if ('Phase' in kind) return `This moved from ${from} to ${to(kind.Phase.to)}.`;
  if ('Return' in kind) return `This moved back from ${from} to ${to(kind.Return.to)}.`;
  if ('Convert' in kind) return 'This closed and its work continued as a new proposal.';
  return 'The proposed changes were made.';
}

function failedSentence(kind: ProposalKind, from: string, to: (id: string) => string): string {
  if ('Phase' in kind)
    return `This stayed in ${from}; the move to ${to(kind.Phase.to)} didn't pass.`;
  if ('Return' in kind) return `This stayed in ${from}; going back didn't pass.`;
  return "This didn't pass, so nothing changed.";
}

/** The sheet's lead line: what passing would do, or what it did. */
export function resultSentence(
  p: Pick<Proposal, 'kind' | 'status'>,
  from: string,
  to: (id: string) => string,
): string {
  if (p.status === 'Passed') return passedSentence(p.kind, from, to);
  if (p.status === 'Rejected') return failedSentence(p.kind, from, to);
  return outcomeSentence(p.kind, from, to);
}
