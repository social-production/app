import { t } from '#lib/i18n/index.ts';
/** web's platform board (ADR 0095): moderators and candidates, replayed in the directory. */

/** web's `confidenceStandingState`: a moderator is `active` or in `grace`; a candidate is
 * `qualifying` once it has the votes, `below_threshold` until then. */
export type BoardStanding = 'active' | 'grace' | 'qualifying' | 'below_threshold';

export interface BoardMember {
  agent: string;
  moderator: boolean;
  standing: BoardStanding;
  graceEndsAt: number | null;
  yes: number;
  no: number;
  /** yes / (yes + no), or null with no votes. */
  ratio: number | null;
  votesRequired: number;
  weeklyActive: number;
  /** The viewer's counted vote: 1 yes, -1 no, 0 none (web's `confidenceActiveVote`). */
  myVote: 1 | -1 | 0;
}

export interface Board {
  weeklyActive: number;
  quorum: number;
  moderators: BoardMember[];
  candidates: BoardMember[];
  /** web's `moderatorCandidacyOptions.viewerState`. */
  viewerState: 'member' | 'candidate' | null;
}

/** web's `_meets_standing_threshold` at 66% (sp_board::APPROVAL_PERCENT). */
export const qualifies = (yes: number, no: number, quorum: number): boolean =>
  yes + no >= quorum && (yes + no === 0 || yes * 100 >= 66 * (yes + no));

const DAY_MS = 24 * 60 * 60 * 1000;
const percent = (m: BoardMember) => (m.ratio === null ? null : Math.round(m.ratio * 100));
const votesOf = (m: BoardMember) => m.yes + m.no;
const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? '' : 's'}`;

/** A row's state chip (Platform and Moderators mockups §2): Seated, Grace · N days, Candidate. */
export function boardChip(m: BoardMember, now = Date.now()): string {
  if (m.standing === 'active') return 'Seated';
  if (m.standing === 'grace') {
    const days =
      m.graceEndsAt === null ? 7 : Math.max(1, Math.ceil((m.graceEndsAt - now) / DAY_MS));
    return `${t('Grace')} · ${plural(days, 'day')}`;
  }
  return m.standing === 'qualifying' ? 'Ready' : 'Candidate';
}

/** A row's one line: how the person stands against this week's quorum. */
export function boardLine(m: BoardMember, quorum: number): string {
  const votes = votesOf(m);
  const pct = percent(m);
  if (m.standing === 'active') {
    if (votes === 0) return quorum === 0 ? t('Seated straight away') : t('No votes yet');
    return `${pct}% yes from ${plural(votes, 'vote')}`;
  }
  if (m.standing === 'grace') {
    return `${pct ?? 0}% yes, but ${votes} of ${quorum} votes this week`;
  }
  if (m.standing === 'qualifying') return t('Has the votes; seated at the next count');
  return votes ? `${votes} of ${quorum} votes · ${pct}% yes` : `0 of ${quorum} votes`;
}

/** How far a candidate is towards the votes a seat needs, 0 to 100. */
export const boardProgress = (m: BoardMember, quorum: number): number =>
  quorum === 0 ? 100 : Math.min(100, Math.round((100 * votesOf(m)) / quorum));

/** The viewer first, then by name. */
export function boardOrder(
  list: BoardMember[],
  me: string,
  name: (m: BoardMember) => string,
): BoardMember[] {
  return [...list].sort(
    (a, b) =>
      Number(b.agent === me) - Number(a.agent === me) ||
      name(a).localeCompare(name(b), undefined, { sensitivity: 'base' }),
  );
}

/** A project tagged #platform takes a network vote: any Full member can vote (ADR 0095). */
export const isNetworkVote = (tags: readonly string[]): boolean => tags.includes('#platform');

/** Who may vote on a space's proposals: its members, and on a #platform project any Full
 * member (ADR 0109; the zome refuses anyone else's vote with its reason). */
export const canVoteOn = (
  p: { amMember: boolean; charter: { tags: string[] } | null } | undefined,
): boolean => !!p?.amMember || isNetworkVote(p?.charter?.tags ?? []);
