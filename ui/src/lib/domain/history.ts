/**
 * web's History tab (`ProjectHistoryTab`, `DecisionHistoryList`, `DecisionHistoryCard`): every
 * decision on a project or event, open ones first in their own group. Built from the
 * governance zome's proposals and plans, as web-backend's `_build_project_history` builds it
 * from its request tables.
 */
import { APPROVAL_PERCENT, type Proposal, type VoteChoice } from './governance.ts';
import type { PlanKind, PlanView } from './plans.ts';

export type HistoryStatus = 'open' | 'approved' | 'rejected';

export interface HistoryEntry {
  id: string;
  /** The proposal itself (decisions with web's vote sheet); plans open their own sheet. */
  proposal: Proposal | null;
  plan: PlanView | null;
  kindLabel: string;
  author: string;
  createdAt: number;
  status: HistoryStatus;
  yes: number;
  no: number;
  required: number;
  population: number;
  myVote: VoteChoice | null;
}

function statusOf(status: Proposal['status']): HistoryStatus {
  if (status === 'Passed') return 'approved';
  return status === 'Rejected' ? 'rejected' : 'open';
}

/** web-backend's phase decision label: Return to …, Close project, Advance to …. */
function proposalKindLabel(p: Proposal, stageLabel: (id: string) => string): string | null {
  const kind = p.kind;
  if ('Update' in kind) return 'Update decision';
  if ('Edit' in kind) return 'Edit decision';
  if ('Return' in kind) return `Return to ${stageLabel(kind.Return.to)}`;
  if ('Phase' in kind) {
    const to = stageLabel(kind.Phase.to);
    return to === 'Closed' ? 'Close project' : `Advance to ${to}`;
  }
  return null;
}

/** web-backend's plan decision label. */
function planKindLabel(kind: PlanKind, title: string): string {
  if (kind === 'Distribution' || kind === 'Access') return `Distribution plan · ${title}`;
  if (kind === 'Production' || kind === 'Organisation') return `Production plan · ${title}`;
  return `Plan · ${title}`;
}

/** Every decision, newest first. Member removals (scopes) are not project history. */
export function historyEntries(
  proposals: Proposal[],
  plans: PlanView[],
  stageLabel: (id: string) => string,
): HistoryEntry[] {
  const out: HistoryEntry[] = [];
  for (const p of proposals) {
    const kindLabel = proposalKindLabel(p, stageLabel);
    if (!kindLabel) continue;
    out.push({
      id: p.id,
      proposal: p,
      plan: null,
      kindLabel,
      author: p.author,
      createdAt: p.createdAt,
      status: statusOf(p.status),
      yes: p.yes,
      no: p.no,
      required: p.required,
      population: p.population,
      myVote: p.myVote,
    });
  }
  for (const v of plans) {
    out.push({
      id: v.id,
      proposal: null,
      plan: v,
      kindLabel: planKindLabel(v.plan.kind, v.plan.title),
      author: v.author,
      createdAt: v.createdAt,
      status: statusOf(v.tally.status),
      yes: v.tally.yes,
      no: v.tally.no,
      required: v.tally.required,
      population: v.tally.population,
      myVote: v.tally.myVote,
    });
  }
  return out.sort((a, b) => b.createdAt - a.createdAt);
}

/** web's `minimumAdditionalYesVotesNeeded`: null when it can no longer pass. */
export function yesStillNeeded(e: HistoryEntry, percent = APPROVAL_PERCENT): number | null {
  const total = e.yes + e.no;
  const left = Math.max(e.population - total, 0);
  if (e.required <= 0) return 0;
  for (let more = 0; more <= left; more++) {
    const after = total + more;
    const quorumGap = Math.max(e.required - after, 0);
    if (more + quorumGap > left) continue;
    const final = after + quorumGap;
    if (final > 0 && (e.yes + more) * 100 >= percent * final) return more;
  }
  return null;
}

/** web's DecisionHistoryCard `requirementLabel`. */
export function requirementLabel(e: HistoryEntry): string {
  if (e.status !== 'open') return `${APPROVAL_PERCENT}% approval needed`;
  const needed = yesStillNeeded(e) ?? 0;
  const left = Math.max(e.population - e.yes - e.no, 0);
  const need = needed > 0 ? `Needs ${needed} more yes` : 'Enough yes votes so far';
  const waiting =
    left > 0 ? ` · ${left} ${left === 1 ? 'person has' : 'people have'} not voted` : '';
  return `${need} (${APPROVAL_PERCENT}% approval)${waiting}`;
}

/** web's `historyVoteSummary` (its approval share rounded; web prints the raw float). */
export function voteSummaryLabel(e: HistoryEntry): string {
  const total = e.yes + e.no;
  const percent = total > 0 ? Math.round((e.yes / total) * 100) : 0;
  const base = `${percent}% approval · ${e.yes} yes · ${e.no} no`;
  if (e.status === 'open') return base;
  const cast = total === 1 ? 'vote' : 'votes';
  const quorum = e.required === 1 ? 'vote' : 'votes';
  return `${base} · ${total} ${cast} cast out of ${e.required} quorum ${quorum}`;
}

/** web's DecisionHistoryCard `statusLabel`. */
export function statusLabel(status: HistoryStatus): string {
  if (status === 'approved') return 'Approved';
  return status === 'rejected' ? 'Rejected' : 'Active';
}
