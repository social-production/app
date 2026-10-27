/** Links between projects (web's Links tab, ADR 0042). Each project votes on its own half. */

import { APPROVAL_PERCENT, type VoteChoice } from './governance.ts';
import type { CharterMode } from './projects.ts';

export type SideStatus = 'Open' | 'Approved' | 'Rejected';

/** One project's half of a request with its votes (web's ProjectManualLinkVoteState). */
export interface LinkHalf {
  id: string;
  author: string;
  createdAt: number;
  relationship: string;
  summary: string;
  yes: number;
  no: number;
  population: number;
  required: number;
  status: SideStatus;
  myVote: VoteChoice | null;
}

/** One side of a request: the project, its half (null until it arrives) and the viewer. */
export interface LinkSide {
  projectId: string;
  projectTitle: string;
  half: LinkHalf | null;
  /** The viewer is a member of this project and hasn't voted on this open half. */
  canVote: boolean;
}

/** web's DetailLinkTargetDetail: the other project as its card shows it. */
export interface LinkTarget {
  id: string;
  title: string;
  description: string;
  mode: CharterMode;
  stageLabel: string;
  locationLabel: string;
  memberCount: number | null;
}

/** web's DetailLinkRequest, seen from this project. */
export interface LinkRequest {
  id: string;
  sever: boolean;
  relationship: string;
  summary: string;
  proposedBy: string;
  createdAt: number;
  status: SideStatus;
  /** This project's half was proposed here (web's "source"). */
  proposedHere: boolean;
  target: LinkTarget;
  thisSide: LinkSide;
  otherSide: LinkSide;
}

/** An approved link; `sever` is its open sever vote. */
export interface ProjectLink {
  id: string;
  create: LinkRequest;
  sever: LinkRequest | null;
}

/** web's DetailLinksFrameData. */
export interface LinksFrame {
  active: ProjectLink[];
  historical: ProjectLink[];
  pending: LinkRequest[];
  past: LinkRequest[];
  canPropose: boolean;
}

export const LINKS_INTRO = 'Use links to connect this project with related projects and events.';

function percent(half: Pick<LinkHalf, 'yes' | 'no'>): number {
  const total = half.yes + half.no;
  return total > 0 ? Math.round((half.yes / total) * 100) : 0;
}

/** web's `formatProjectVoteSummary`. */
export function voteSummary(half: LinkHalf | null): string {
  if (!half) return '0% approval · 0 yes · 0 no';
  return `${percent(half)}% approval · ${half.yes} yes · ${half.no} no`;
}

/** web's `minimumAdditionalYesVotesNeeded`: the fewest more Yes votes that pass it. */
function yesNeeded(half: LinkHalf): number | null {
  const total = half.yes + half.no;
  const left = Math.max(half.population - total, 0);
  if (half.required <= 0) return 0;
  for (let more = 0; more <= left; more++) {
    const after = total + more;
    const forQuorum = Math.max(half.required - after, 0);
    if (more + forQuorum > left) continue;
    const finalTotal = after + forQuorum;
    if (finalTotal > 0 && (half.yes + more) * 100 >= APPROVAL_PERCENT * finalTotal) return more;
  }
  return null;
}

/** web's `formatProjectVoteRequirement`. */
export function voteRequirement(half: LinkHalf): string {
  const needed = yesNeeded(half) ?? 0;
  const left = Math.max(half.population - half.yes - half.no, 0);
  const need = needed > 0 ? `Needs ${needed} more yes` : 'Enough yes votes so far';
  const waiting =
    left > 0 ? ` · ${left} ${left === 1 ? 'person has' : 'people have'} not voted` : '';
  return `${need} (${APPROVAL_PERCENT}% approval)${waiting}`;
}

/** web's `resultNote`; a half not received yet is an app addition (no server delivers it). */
export function resultNote(half: LinkHalf | null): string {
  if (!half) return 'Waiting for this project to receive the request.';
  if (half.status === 'Approved') return 'Approved on this side.';
  return half.status === 'Open'
    ? 'Waiting for more approvals.'
    : 'This side can no longer approve the request.';
}

/** web's `side_vote_label`. */
function sideVoteLabel(half: LinkHalf | null): string {
  return half ? `${percent(half)}% · ${half.yes} yes / ${half.no} no` : '0% · 0 yes / 0 no';
}

/** web's `sideVoteLines`: proposing project first. */
export function sideVoteLines(r: LinkRequest): string[] {
  const [source, target] = r.proposedHere ? [r.thisSide, r.otherSide] : [r.otherSide, r.thisSide];
  return [source, target].map((s) => `${s.projectTitle}: ${sideVoteLabel(s.half)}`);
}

/** web's `_governance_tally` label: both sides' votes together. */
export function tallyLabel(r: LinkRequest): string {
  const halves = [r.thisSide.half, r.otherSide.half];
  const yes = halves.reduce((n, h) => n + (h?.yes ?? 0), 0);
  const no = halves.reduce((n, h) => n + (h?.no ?? 0), 0);
  return yes + no === 0 ? 'No votes yet' : `${percent({ yes, no })}% · ${yes} yes / ${no} no`;
}

/** web's status labels ("Open", "Approved", "Rejected"; "Sever open" while a sever runs). */
export function statusLabel(r: LinkRequest): string {
  return r.sever && r.status === 'Open' ? 'Sever open' : r.status;
}

/** web's `detailMetaRows` (placeholder labels such as "TBD" are left out). */
export function targetMetaRows(t: LinkTarget): string[] {
  const meaningful = (v: string) => {
    const s = v.trim();
    return s && !['tbd', 'location tbd', 'time tbd'].includes(s.toLowerCase()) ? s : null;
  };
  const rows = [meaningful(t.stageLabel), meaningful(t.locationLabel)].filter(
    (r): r is string => !!r,
  );
  if (t.memberCount !== null) rows.push(`${t.memberCount} members`);
  return rows;
}
