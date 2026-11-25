import { plural, t } from '#lib/i18n/index.ts';
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
  /** People active so far this week (shown beside last week's; votes use last week's). */
  thisWeek?: number | undefined;
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
const votesOf = (m: BoardMember) => m.yes + m.no;

/** A row's chip, only when the section heading doesn't already say it: Grace · N days for a
 * moderator short of votes, Ready (Has the votes) for a candidate who has them. */
export function boardChip(m: BoardMember, now = Date.now()): string | null {
  if (m.standing === 'grace') {
    const days =
      m.graceEndsAt === null ? 7 : Math.max(1, Math.ceil((m.graceEndsAt - now) / DAY_MS));
    return `${t('Grace')} · ${plural(days, '{n} day', '{n} days')}`;
  }
  return m.standing === 'qualifying' ? t('Has the votes') : null;
}

/** A row's one line, in words: "8 of 9 yes", "2 votes · needs 3 more", "Seated straight away". */
export function boardLine(m: BoardMember, quorum: number): string {
  const votes = votesOf(m);
  if (m.standing === 'active' && votes === 0)
    return quorum === 0 ? t('Seated straight away') : t('No votes yet');
  if (m.standing === 'qualifying') return t('Seated at the next count');
  const tally = t('{yes} of {n} yes', { yes: m.yes, n: votes });
  if (m.moderator && m.standing !== 'grace') return tally;
  const short = Math.max(0, quorum - votes);
  if (short === 0) return tally;
  const more = plural(short, 'needs {n} more vote', 'needs {n} more votes');
  return votes ? `${tally} · ${more}` : more;
}

/** The row's bar, 0 to 100: a moderator's share of yes, a candidate's way to the votes a seat
 * needs; none before anyone has voted on a moderator. */
export function boardBar(m: BoardMember, quorum: number): number | null {
  const votes = votesOf(m);
  if (m.moderator) return votes ? Math.round((100 * m.yes) / votes) : null;
  return quorum === 0 ? 100 : Math.min(100, Math.round((100 * votes) / quorum));
}

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
