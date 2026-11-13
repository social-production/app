import { qualifies, type Board, type BoardMember } from '#lib/domain/board.ts';
import {
  callZome,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { myAgent } from './people.ts';

/** web's board commands (ADR 0095): volunteer, withdraw, vote; the board is replayed. */
const identity = <T>(fn: string, payload?: unknown) =>
  callZome<T>('directory', 'identity', fn, payload);

interface RowWire {
  agent: AgentPubKey;
  standing: 'candidate' | 'moderator' | 'grace';
  grace_ends: number | null;
  yes: number;
  no: number;
  my_vote: boolean | null;
}

interface BoardWire {
  weekly_active: number;
  quorum: number;
  rows: RowWire[];
}

function member(r: RowWire, quorum: number, weeklyActive: number): BoardMember {
  const votes = r.yes + r.no;
  const standing =
    r.standing === 'moderator'
      ? 'active'
      : r.standing === 'grace'
        ? 'grace'
        : qualifies(r.yes, r.no, quorum)
          ? 'qualifying'
          : 'below_threshold';
  return {
    agent: hashToString(r.agent),
    moderator: r.standing !== 'candidate',
    standing,
    graceEndsAt: r.grace_ends === null ? null : Math.floor(r.grace_ends / 1000),
    yes: r.yes,
    no: r.no,
    ratio: votes ? r.yes / votes : null,
    votesRequired: quorum,
    weeklyActive,
    myVote: r.my_vote === null ? 0 : r.my_vote ? 1 : -1,
  };
}

export async function getBoard(): Promise<Board> {
  const [w, me] = await Promise.all([identity<BoardWire>('get_board'), myAgent().catch(() => '')]);
  const rows = w.rows.map((r) => member(r, w.quorum, w.weekly_active));
  const mine = rows.find((r) => r.agent === me);
  return {
    weeklyActive: w.weekly_active,
    quorum: w.quorum,
    moderators: rows.filter((r) => r.moderator),
    candidates: rows.filter((r) => !r.moderator),
    viewerState: !mine ? null : mine.moderator ? 'member' : 'candidate',
  };
}

/** web's volunteerForBoard (`true`) and removeVolunteer (`false`, withdraw or step down). */
export async function volunteerForBoard(on: boolean): Promise<void> {
  await identity<ActionHash>('volunteer_board', on);
}

/** web's castStandingVote: 1 yes, -1 no, 0 neutral (withdraws the vote). */
export async function voteOnBoard(agent: string, vote: 1 | -1 | 0): Promise<void> {
  const choice = vote === 1 ? 'yes' : vote === -1 ? 'no' : 'neutral';
  await identity<ActionHash>('vote_board', { account: stringToHash(agent), vote: choice });
}
