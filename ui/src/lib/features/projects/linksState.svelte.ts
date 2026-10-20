import type { VoteChoice } from '#lib/domain/governance.ts';
import type { LinkSide, LinksFrame, ProjectLink } from '#lib/domain/links.ts';
import type { Project } from '#lib/domain/projects.ts';
import { explain } from '#lib/services/errors.ts';
import {
  listLinks,
  proposeLink,
  proposeSever,
  searchProjects,
  voteLink,
} from '#lib/services/links.ts';
import { getPerson } from '#lib/services/people.ts';

/** The Links tab of one project (ADR 0042): read when the tab first opens, as on web. */
export class LinksState {
  frame = $state<LinksFrame>();
  names = $state<Record<string, string>>({});
  error = $state<string>();
  /** The link or request an action is running for. */
  pendingId = $state<string | null>(null);
  saving = $state(false);
  composerMessage = $state<string>();

  constructor(private readonly project: () => Project | undefined) {}

  async load(): Promise<void> {
    const here = this.project();
    if (!here) return;
    try {
      const frame = await listLinks(here);
      const all = [...frame.pending, ...frame.past];
      const missing = all
        .map((r) => r.proposedBy)
        .filter((a, i, list) => list.indexOf(a) === i && !(a in this.names));
      const people = await Promise.all(missing.map(getPerson));
      const names = { ...this.names };
      for (const p of people) names[p.agent] = p.profile?.name ?? 'Unnamed';
      this.names = names;
      this.frame = frame;
      this.error = undefined;
    } catch (e) {
      this.error = explain(e);
    }
  }

  search = (q: string) => searchProjects(q, this.project()?.id ?? '');

  /** web's "Create link vote". Returns true when filed in both projects. */
  async propose(targetId: string, summary: string): Promise<boolean> {
    const here = this.project();
    if (!here) return false;
    const open = this.frame?.pending.some((r) => r.target.id === targetId);
    const linked = this.frame?.active.some((l) => l.create.target.id === targetId);
    if (open || linked) {
      this.composerMessage = linked
        ? 'An active link already exists between these records.'
        : 'A link vote with that project is already open.';
      return false;
    }
    this.saving = true;
    this.composerMessage = undefined;
    try {
      await proposeLink(here, targetId, summary);
      await this.load();
      return true;
    } catch (e) {
      this.composerMessage = explain(e);
      return false;
    } finally {
      this.saving = false;
    }
  }

  sever(link: ProjectLink): Promise<void> {
    const here = this.project();
    return here ? this.run(link.id, () => proposeSever(here, link)) : Promise.resolve();
  }

  vote(requestId: string, side: LinkSide, choice: VoteChoice): Promise<void> {
    const half = side.half;
    return half
      ? this.run(requestId, () => voteLink(side.projectId, half.id, choice))
      : Promise.resolve();
  }

  private async run(id: string, action: () => Promise<void>): Promise<void> {
    this.pendingId = id;
    try {
      await action();
      await this.load();
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.pendingId = null;
    }
  }
}
