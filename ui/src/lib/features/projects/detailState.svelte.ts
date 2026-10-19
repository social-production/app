import {
  isPhaseChange,
  type Proposal,
  type ProposalKind,
  type VoteChoice,
} from '#lib/domain/governance.ts';
import type { Person } from '#lib/domain/people.ts';
import type { Plan, PlanBoard, PlanRating } from '#lib/domain/plans.ts';
import type { CharterDraft, Project, Stance } from '#lib/domain/projects.ts';
import { explain } from '#lib/services/errors.ts';
import {
  castVote,
  challengeCount,
  closeProposal,
  listProposals,
  propose,
} from '#lib/services/governance.ts';
import { getPerson } from '#lib/services/people.ts';
import { listPlans, ratePlan as rate, submitPlan as submit } from '#lib/services/plans.ts';
import { getProject, joinProject, setSignal } from '#lib/services/projects.ts';

/** One project page: the project, its people and proposals, and the viewer's actions on it. */
export class DetailState {
  project = $state<Project>();
  members = $state<Person[]>([]);
  proposals = $state<Proposal[]>([]);
  plans = $state<PlanBoard>();
  /** Display names by agent key (creator, members, proposers, update authors). */
  names = $state<Record<string, string>>({});
  error = $state<string>();
  joining = $state(false);
  saving = $state(false);
  sheetMessage = $state<string>();

  constructor(readonly id: string) {}

  get canPropose(): boolean {
    return !!this.project?.amMember && this.project.charter?.mode !== 'personal-service';
  }

  /** Open proposals, the viewer's unvoted ones first. */
  get openProposals(): Proposal[] {
    return this.proposals
      .filter((p) => p.status === 'Open')
      .sort((a, b) => Number(!!a.myVote) - Number(!!b.myVote));
  }

  /** web's `hasOpenPhaseChangeVotes`: a move or return is still being decided. */
  get phaseVoteOpen(): boolean {
    return this.proposals.some((p) => p.status === 'Open' && isPhaseChange(p.kind));
  }

  /** Decisions whose count left members out, while a challenge still counts (ADR 0040). */
  get challengeable(): Proposal[] {
    return this.proposals.filter((p) => p.canChallenge);
  }

  phaseLabel = (id: string): string => this.project?.phases.find((p) => p.id === id)?.label ?? id;

  /** First load: a project just created elsewhere may take a few seconds to reach this node. */
  async open(): Promise<void> {
    for (let attempt = 1; ; attempt++) {
      try {
        await this.fetch();
        return;
      } catch (e) {
        if (attempt >= 15) {
          this.error = explain(e);
          return;
        }
        await new Promise((resolve) => setTimeout(resolve, 2000));
      }
    }
  }

  /** Refresh after an action; failures show as the page's error. */
  async load(): Promise<void> {
    try {
      await this.fetch();
    } catch (e) {
      this.error = explain(e);
    }
  }

  private read() {
    return Promise.all([getProject(this.id), listProposals(this.id), listPlans(this.id)]);
  }

  private async fetch(): Promise<void> {
    let [project, proposals, plans] = await this.read();
    // A vote that decided a proposal may have been cast before the voter saw every other vote;
    // whoever loads the page next writes the Outcome (ADR 0033).
    const settled = project.amMember
      ? [
          ...proposals.filter((p) => p.settled).map((p) => p.id),
          ...plans.plans.filter((p) => p.tally.settled).map((p) => p.id),
        ]
      : [];
    if (settled.length > 0) {
      await Promise.all(settled.map((id) => closeProposal(this.id, id)));
      [project, proposals, plans] = await this.read();
    }
    const members = await Promise.all(project.members.map(getPerson));
    this.project = project;
    this.members = members;
    this.proposals = proposals;
    this.plans = plans;
    this.error = undefined;
    const keys = [
      project.creator,
      ...proposals.map((p) => p.author),
      ...plans.plans.map((p) => p.author),
      ...project.updates.map((u) => u.author),
    ];
    await this.loadNames([...members.map((m) => m.agent), ...keys], members);
  }

  private async loadNames(agents: string[], known: Person[]): Promise<void> {
    const names: Record<string, string> = { ...this.names };
    for (const p of known) names[p.agent] = p.profile?.name ?? 'Unnamed';
    const missing = agents.filter((a, i) => agents.indexOf(a) === i && !(a in names));
    for (const p of await Promise.all(missing.map(getPerson)))
      names[p.agent] = p.profile?.name ?? 'Unnamed';
    this.names = names;
  }

  async toggleJoin(): Promise<void> {
    if (!this.project) return;
    this.joining = true;
    try {
      await joinProject(this.id, !this.project.amMember);
      await this.load();
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.joining = false;
    }
  }

  signal(next: Stance | null): Promise<void> {
    return setSignal(this.id, next);
  }

  async vote(proposal: Proposal, choice: VoteChoice): Promise<void> {
    await castVote(this.id, proposal.id, choice);
    await this.load();
  }

  /** web's ProposeEditSheet: only fields that change are proposed. Returns true when filed. */
  proposeEdit(draft: CharterDraft): Promise<boolean> {
    const c = this.project?.charter;
    if (!c) return Promise.resolve(false);
    const changed = (next: string, now: string): string | null =>
      next.trim() === now.trim() ? null : next;
    const edit = {
      title: changed(draft.title, c.title),
      description: changed(draft.description, c.description),
      location: changed(draft.location, c.location_label),
    };
    if (edit.title === null && edit.description === null && edit.location === null) {
      this.sheetMessage = 'Nothing has changed.';
      return Promise.resolve(false);
    }
    return this.file({ Edit: edit });
  }

  proposeUpdate(body: string): Promise<boolean> {
    return this.file({ Update: { body } });
  }

  proposeNextPhase(): Promise<boolean> {
    const next = this.project?.nextPhase;
    return next ? this.file({ Phase: { to: next.id, plan: null } }) : Promise.resolve(false);
  }

  /** web's return request (`revert-requests`). Returns true when filed. */
  proposeReturn(to: string, reason: string): Promise<boolean> {
    return this.file({ Return: { to, reason } });
  }

  async challenge(proposal: string): Promise<void> {
    try {
      await challengeCount(this.id, proposal);
      await this.load();
    } catch (e) {
      this.error = explain(e);
    }
  }

  /** web's plan wizard submit. Returns true when filed. */
  async submitPlan(plan: Plan): Promise<boolean> {
    this.saving = true;
    this.sheetMessage = undefined;
    try {
      await submit(this.id, plan);
      await this.load();
      return true;
    } catch (e) {
      this.sheetMessage = explain(e);
      return false;
    } finally {
      this.saving = false;
    }
  }

  /** Ratings change the leader only, so the page refreshes quietly behind them. */
  async ratePlan(plan: string, criterion: string, rating: PlanRating | null): Promise<void> {
    try {
      await rate(this.id, plan, criterion, rating);
      await this.load();
    } catch (e) {
      this.error = explain(e);
    }
  }

  async votePlan(plan: string, choice: VoteChoice): Promise<void> {
    try {
      await castVote(this.id, plan, choice);
      await this.load();
    } catch (e) {
      this.error = explain(e);
    }
  }

  private async file(kind: ProposalKind): Promise<boolean> {
    this.saving = true;
    this.sheetMessage = undefined;
    try {
      await propose(this.id, kind);
      await this.load();
      return true;
    } catch (e) {
      this.sheetMessage = explain(e);
      return false;
    } finally {
      this.saving = false;
    }
  }
}
