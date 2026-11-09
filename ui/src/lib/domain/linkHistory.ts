/**
 * Link create and sever votes in the History tab (web-backend's
 * `build_link_decision_history_entries`). Each becomes a generic-shaped proposal, as software
 * votes do (`softwareVotes.ts`), so the History card and vote sheet show it; its tally and
 * vote are this project's half, its status the whole request's.
 */
import type { Proposal } from './governance.ts';
import type { HistoryEntry, HistoryStatus } from './history.ts';
import { allRequests, sideLine, type LinkRequest, type LinksFrame } from './links.ts';

function statusOf(r: LinkRequest): HistoryStatus {
  if (r.status === 'Approved') return 'approved';
  return r.status === 'Rejected' ? 'rejected' : 'open';
}

function proposalOf(r: LinkRequest): Proposal | null {
  const half = r.thisSide.half;
  if (!half) return null;
  const status = statusOf(r);
  return {
    id: half.id,
    author: r.proposedBy,
    // web's `historyEntryToVoteItem` for `payload.type === 'link'`.
    kind: {
      Software: {
        label: r.sever ? 'Sever' : 'Link',
        title: r.target.title,
        description: `${sideLine(r.thisSide)} · ${sideLine(r.otherSide)}`,
        reason: r.summary,
      },
    },
    createdAt: r.createdAt,
    yes: half.yes,
    no: half.no,
    neutral: 0,
    population: half.population,
    required: half.required,
    // Votes are cast only while the whole request is still open.
    status: status === 'open' ? 'Open' : status === 'approved' ? 'Passed' : 'Rejected',
    closedAt: null,
    myVote: half.myVote,
    settled: false,
    added: 0,
    missing: 0,
    challengeUntil: null,
    canChallenge: false,
  };
}

/** The request each History entry votes on, by the entry's id (this project's half). */
export function linkRequestsById(frame: LinksFrame | undefined): Map<string, LinkRequest> {
  const out = new Map<string, LinkRequest>();
  for (const r of frame ? allRequests(frame) : []) {
    if (r.thisSide.half) out.set(r.thisSide.half.id, r);
  }
  return out;
}

/** web's "Link decision" / "Sever link decision" entries, newest first. */
export function linkHistory(frame: LinksFrame | undefined): HistoryEntry[] {
  const out: HistoryEntry[] = [];
  for (const r of frame ? allRequests(frame) : []) {
    const p = proposalOf(r);
    if (!p) continue;
    out.push({
      id: p.id,
      proposal: p,
      plan: null,
      kindLabel: r.sever ? 'Sever link decision' : 'Link decision',
      author: p.author,
      createdAt: p.createdAt,
      status: statusOf(r),
      yes: p.yes,
      no: p.no,
      required: p.required,
      population: p.population,
      myVote: p.myVote,
    });
  }
  return out.sort((a, b) => b.createdAt - a.createdAt);
}
