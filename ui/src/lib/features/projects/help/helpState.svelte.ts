import type { Activity, ActivityRole } from '#lib/domain/activities.ts';
import type { Project, Stance } from '#lib/domain/projects.ts';
import { explain } from '#lib/services/errors.ts';
import { commitToRole, helpActivity } from '#lib/services/helpRequests.ts';
import { getPerson } from '#lib/services/people.ts';
import { getProject, setSignal } from '#lib/services/projects.ts';
import { requireViewer } from '#lib/stores/authNotice.svelte.ts';

/** One help request page (web's HelpRequestPageData): the space, its activity and names. */
export class HelpState {
  project = $state<Project>();
  activity = $state<Activity | null>(null);
  names = $state<Record<string, string>>({});
  error = $state<string>();
  /** The role being signed up for or left (web's `rolePending`). */
  pending = $state<number | null>(null);
  roleMessage = $state('');

  constructor(readonly id: string) {}

  /** First load: a request just created may take a few seconds to reach this node. */
  async open(): Promise<void> {
    for (let attempt = 1; ; attempt++) {
      try {
        await this.fetch();
        if (this.activity || attempt >= 15) return;
      } catch (e) {
        if (attempt >= 15) {
          this.error = explain(e);
          return;
        }
      }
      await new Promise((resolve) => setTimeout(resolve, 2000));
    }
  }

  private async fetch(): Promise<void> {
    const [project, activity] = await Promise.all([getProject(this.id), helpActivity(this.id)]);
    this.project = project;
    this.activity = activity;
    this.error = undefined;
    const agents = [project.creator, ...(activity?.roles.flatMap((r) => r.people) ?? [])];
    const missing = agents.filter((a, i) => agents.indexOf(a) === i && !(a in this.names));
    const people = await Promise.all(missing.map(getPerson));
    this.names = {
      ...this.names,
      ...Object.fromEntries(people.map((p) => [p.agent, p.profile?.name ?? 'Unnamed'])),
    };
  }

  /** web's roleHasOpenCapacity. */
  hasRoom(role: ActivityRole): boolean {
    return role.maximum === null || role.people.length < role.maximum;
  }

  /** web's handleRoleCommitment: sign up, or leave the role you hold. */
  async toggle(role: ActivityRole): Promise<void> {
    const activity = this.activity;
    if (!activity || this.pending !== null || !requireViewer()) return;
    this.pending = role.index;
    this.roleMessage = '';
    try {
      const mine = activity.myRole === role.index;
      await commitToRole(this.id, activity.id, mine ? null : role.index, !!this.project?.amMember);
      await this.fetch();
    } catch (e) {
      this.roleMessage = explain(e);
    } finally {
      this.pending = null;
    }
  }

  /** web's up/down vote: the request's Support/Oppose signal (ADR 0067). */
  async vote(next: Stance | null): Promise<void> {
    if (!requireViewer()) throw new Error('signed out');
    await setSignal(this.id, next);
    void this.fetch().catch(() => undefined);
  }
}
