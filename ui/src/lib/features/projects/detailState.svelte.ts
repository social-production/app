import {
  isPhaseChange,
  type Proposal,
  type ProposalKind,
  type VoteChoice,
} from '#lib/domain/governance.ts';
import type { Person } from '#lib/domain/people.ts';
import type { Plan, PlanBoard, PlanRating } from '#lib/domain/plans.ts';
import { charterEdit, type CharterDraft, type Project, type Stance } from '#lib/domain/projects.ts';
import { isSoftwareProject } from '#lib/domain/software.ts';
import type { Importance, ProjectValue } from '#lib/domain/values.ts';
import { explain } from '#lib/services/errors.ts';
import {
  castVote,
  challengeCount,
  closeProposal,
  listProposals,
  propose,
} from '#lib/services/governance.ts';
import { getPerson, myAgent } from '#lib/services/people.ts';
import { listPlans, ratePlan as rate, submitPlan as submit } from '#lib/services/plans.ts';
import { getProject, joinProject, setSignal } from '#lib/services/projects.ts';
import { addValue, listValues, rateValue } from '#lib/services/values.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';

import { tellDecisions } from './outcomeNotices.ts';
import { tellMembers } from './phaseNotice.ts';
import { SoftwareState } from './software/softwareState.svelte.ts';

/** One project page: the project, its people and proposals, and the viewer's actions on it. */
export class DetailState {
  project = $state<Project>();
  members = $state<Person[]>([]);
  proposals = $state<Proposal[]>([]);
  plans = $state<PlanBoard>();
  values = $state<ProjectValue[]>([]);
  /** Display names by agent key (creator, members, proposers, update authors). */
  names = $state<Record<string, string>>({});
  /** Whether the viewer's votes count: everyone's do unless found to be a bot (ADR 0118). */
  viewerVotes = $state<boolean | null>(null);
  error = $state<string>();
  joining = $state(false);
  saving = $state(false);
  sheetMessage = $state<string>();

  /** A software project's pull requests and their votes (ADR 0086), read with the page. */
  readonly software: SoftwareState;
  constructor(readonly id: string) {
    this.software = new SoftwareState(id);
  }

  /** Leaving an event asks first (web's confirmEventMembershipToggle). */
  confirmingLeave = $state(false);

  get canPropose(): boolean {
    const p = this.project;
    // Organizer-controlled events take changes from organizers only (ADR 0061).
    const organized = p?.governance === 'organizer_controlled';
    return !!p?.amMember && (!organized || p.amOrganizer);
  }

  /** Join at once; leaving always waits for the confirm dialog (owner, 2026-11-18: one tap
   * on the ⋯ menu's Leave took a member out of a project). */
  requestToggleJoin(): void {
    if (this.project?.amMember) {
      this.confirmingLeave = true;
      return;
    }
    void this.toggleJoin();
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
    return Promise.all([
      getProject(this.id),
      listProposals(this.id),
      listPlans(this.id),
      listValues(this.id),
    ]);
  }

  private async fetch(): Promise<void> {
    let [project, proposals, plans, values] = await this.read();
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
      [project, proposals, plans, values] = await this.read();
    }
    const members = await Promise.all(project.members.map(getPerson));
    if (this.viewerVotes === null && project.amMember) void this.loadViewer();
    this.project = project;
    this.members = members;
    this.proposals = proposals;
    this.plans = plans;
    this.values = values;
    this.error = undefined;
    tellDecisions(this, settled, settled.length > 0);
    if (isSoftwareProject(project.charter)) void this.software.load();
    const keys = [
      project.creator,
      ...proposals.map((p) => p.author),
      ...plans.plans.map((p) => p.author),
      ...values.map((v) => v.author),
      ...project.updates.map((u) => u.author),
    ];
    await this.loadNames([...members.map((m) => m.agent), ...keys], members);
  }

  private async loadViewer(): Promise<void> {
    try {
      this.viewerVotes = (await getPerson(await myAgent())).standing.level !== 'Paused';
    } catch {
      // Unknown standing shows no note.
    }
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
    if (!this.project || !requireViewer('Sign in to join this project.')) return;
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

  /** Guests see web's sign-in notice; the strip rolls back. */
  signal(next: Stance | null): Promise<void> {
    if (!requireViewer('Sign in to support this project.')) {
      return Promise.reject(new Error('Sign in to support this project.'));
    }
    return setSignal(this.id, next);
  }

  async vote(proposal: Proposal, choice: VoteChoice): Promise<void> {
    await castVote(this.id, proposal.id, choice);
    await this.load();
    tellDecisions(this, [proposal.id], false);
  }

  /** web's ProposeEditSheet: only fields that change are proposed. Returns true when filed. */
  proposeEdit(draft: CharterDraft): Promise<boolean> {
    const c = this.project?.charter;
    if (!c) return Promise.resolve(false);
    const edit = charterEdit(draft, c);
    const same = !edit.title && !edit.description && !edit.location && !edit.picture;
    if (same && !edit.cover) {
      this.sheetMessage = 'Nothing has changed.';
      return Promise.resolve(false);
    }
    return this.file({ Edit: edit });
  }

  proposeUpdate = (body: string): Promise<boolean> => this.file({ Update: { body } });

  proposeNextPhase(reason: string): Promise<boolean> {
    const next = this.project?.nextPhase;
    return next
      ? this.file({ Phase: { to: next.id, plan: null, reason } })
      : Promise.resolve(false);
  }

  /** Close this and continue its work as a new proposal (ADR 0106). True when filed. */
  proposeConvert = (reason: string): Promise<boolean> => this.file({ Convert: { reason } });

  /** web's return request (`revert-requests`). True when filed. */
  proposeReturn = (to: string, reason: string): Promise<boolean> =>
    this.file({ Return: { to, reason } });

  async challenge(proposal: string): Promise<void> {
    try {
      await challengeCount(this.id, proposal);
      await this.load();
    } catch (e) {
      this.error = explain(e);
    }
  }

  /** web's plan wizard submit. Returns true when filed. */
  submitPlan = (plan: Plan): Promise<boolean> => this.sheetWrite(() => submit(this.id, plan));

  /** web's AddValueSheet submit. Returns true when added. */
  addValue = (label: string): Promise<boolean> => this.sheetWrite(() => addValue(this.id, label));

  rateValue(value: string, importance: Importance): Promise<void> {
    return this.inTurn(() => rateValue(this.id, value, importance));
  }

  /** Ratings change the leader only, so the page refreshes quietly behind them. */
  ratePlan(plan: string, criterion: string, rating: PlanRating | null): Promise<void> {
    return this.inTurn(() => rate(this.id, plan, criterion, rating));
  }

  votePlan(plan: string, choice: VoteChoice): Promise<void> {
    const voted = this.inTurn(() => castVote(this.id, plan, choice));
    return voted.then(() => tellDecisions(this, [], true));
  }

  // Quick taps (three ratings, then Yes) must not write to the chain at the same time: the
  // conductor refuses a write whose chain head moved underneath it. One write at a time.
  private writes: Promise<void> = Promise.resolve();

  private inTurn(write: () => Promise<unknown>): Promise<void> {
    this.writes = this.writes.then(async () => {
      try {
        await write();
        await this.load();
      } catch (e) {
        this.error = explain(e);
      }
    });
    return this.writes;
  }

  private file = (kind: ProposalKind): Promise<boolean> =>
    this.sheetWrite(() => propose(this.id, kind).then(() => tellMembers(this, kind)));

  /** A sheet's write: the page reloads after it; its error shows in the sheet. */
  private async sheetWrite(write: () => Promise<unknown>): Promise<boolean> {
    this.saving = true;
    this.sheetMessage = undefined;
    try {
      await write();
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
