import type { Proposal, VoteChoice } from '#lib/domain/governance.ts';
import { displayName } from '#lib/domain/people.ts';
import { explain } from '#lib/services/errors.ts';
import { castVote, closeProposal, listProposals, propose } from '#lib/services/governance.ts';
import { getPerson } from '#lib/services/people.ts';

/** A scope decision with its proposer's name (web's ScopeProposal). */
export interface ScopeDecision extends Proposal {
  proposer: string;
}

/** web's ScopeSettings for one channel or community: its open and past decisions (ADR 0068). */
export class ScopeGovernance {
  open = $state<ScopeDecision[]>([]);
  past = $state<ScopeDecision[]>([]);
  loaded = $state(false);
  busy = $state(false);
  message = $state<string>();

  constructor(
    private readonly id: string,
    private readonly isMember: () => boolean,
  ) {}

  /** Open decisions the viewer can still vote on (web's viewerCanVote). */
  get votable(): ScopeDecision[] {
    return this.isMember() ? this.open.filter((p) => p.myVote === null) : [];
  }

  async load(): Promise<void> {
    try {
      let proposals = await listProposals(this.id);
      // Whoever loads next writes the Outcome of a decided vote (ADR 0033), as on projects.
      const settled = this.isMember() ? proposals.filter((p) => p.settled) : [];
      if (settled.length > 0) {
        await Promise.all(settled.map((p) => closeProposal(this.id, p.id)));
        proposals = await listProposals(this.id);
      }
      const named = await Promise.all(
        proposals.map(async (p) => ({
          ...p,
          proposer: await getPerson(p.author).then(displayName, () => 'Someone'),
        })),
      );
      this.open = named.filter((p) => p.status === 'Open');
      this.past = named.filter((p) => p.status !== 'Open');
    } catch (e) {
      this.message = explain(e);
    } finally {
      this.loaded = true;
    }
  }

  /** web's "Propose change": a description edit (the name is the handle and stays). */
  proposeDescription(description: string): Promise<boolean> {
    return this.run(() => propose(this.id, { Edit: { title: null, description, location: null } }));
  }

  vote(proposal: string, choice: VoteChoice): Promise<boolean> {
    return this.run(() => castVote(this.id, proposal, choice));
  }

  private async run(write: () => Promise<void>): Promise<boolean> {
    if (this.busy) return false;
    this.busy = true;
    this.message = undefined;
    try {
      await write();
      await this.load();
      return true;
    } catch (e) {
      this.message = explain(e);
      return false;
    } finally {
      this.busy = false;
    }
  }
}

/** web's describeScopeProposal: one line saying what a decision would change. */
export function describeDecision(p: Proposal, memberName: (agent: string) => string): string {
  if ('RemoveMember' in p.kind) return `Remove ${memberName(p.kind.RemoveMember.agent)}`;
  if ('Edit' in p.kind) {
    return `Change: ${p.kind.Edit.description !== null ? 'new description' : 'settings'}`;
  }
  return 'Change: settings';
}
