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
