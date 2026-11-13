import type { Board, BoardMember } from '#lib/domain/board.ts';
import type { Person } from '#lib/domain/people.ts';
import { getBoard, voteOnBoard, volunteerForBoard } from '#lib/services/board.ts';
import { explain } from '#lib/services/errors.ts';
import { getPerson, myAgent } from '#lib/services/people.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';

/** The Platform channel's moderator board (ADR 0095, web's PlatformBoardPanel data). */
export class BoardState {
  board = $state<Board | null>(null);
  people = $state<Record<string, Person>>({});
  /** web's `canVolunteer`: Full standing (web's `ensure_established_voter`). */
  canVolunteer = $state(false);
  error = $state<string>();

  async load(): Promise<void> {
    try {
      const board = await getBoard();
      const agents = [...board.moderators, ...board.candidates].map((m) => m.agent);
      const me = await myAgent().catch(() => '');
      const missing = [...agents, me].filter(
        (a, i, all) => a && all.indexOf(a) === i && !this.people[a],
      );
      const found = await Promise.all(missing.map(getPerson));
      this.people = { ...this.people, ...Object.fromEntries(found.map((p) => [p.agent, p])) };
      this.canVolunteer = !!me && this.people[me]?.standing.level === 'Full';
      this.board = board;
    } catch (e) {
      this.error = explain(e);
    }
  }

  volunteer = async (on: boolean): Promise<void> => {
    if (!requireViewer('Sign in to volunteer as a moderator.')) return;
    try {
      await volunteerForBoard(on);
    } catch (e) {
      throw new Error(explain(e), { cause: e });
    }
    await this.load();
  };

  vote = async (member: BoardMember, vote: 1 | -1 | 0): Promise<void> => {
    if (!requireViewer('Sign in to vote on moderators.')) return;
    try {
      await voteOnBoard(member.agent, vote);
    } catch (e) {
      throw new Error(explain(e), { cause: e });
    }
    await this.load();
  };
}
