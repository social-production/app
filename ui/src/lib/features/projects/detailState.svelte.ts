import type { Person } from '#lib/domain/people.ts';
import type { CharterDraft, Project, Stance } from '#lib/domain/projects.ts';
import { explain } from '#lib/services/errors.ts';
import { getPerson } from '#lib/services/people.ts';
import { getProject, joinProject, setSignal, updateCharter } from '#lib/services/projects.ts';

/** One project page: the project, its people, and the viewer's actions on it. */
export class DetailState {
  project = $state<Project>();
  creator = $state<Person>();
  members = $state<Person[]>([]);
  error = $state<string>();
  joining = $state(false);
  saving = $state(false);
  editMessage = $state<string>();

  readonly id: string;
  private readonly viewer: () => string;

  constructor(id: string, viewer: () => string) {
    this.id = id;
    this.viewer = viewer;
  }

  get isCreator(): boolean {
    return !!this.project && this.project.creator === this.viewer();
  }

  async load(): Promise<void> {
    try {
      const project = await getProject(this.id);
      const [creator, members] = await Promise.all([
        getPerson(project.creator),
        Promise.all(project.members.map(getPerson)),
      ]);
      this.project = project;
      this.creator = creator;
      this.members = members;
    } catch (e) {
      this.error = explain(e);
    }
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

  /** Creator only until M3; returns true when saved. */
  async edit(draft: CharterDraft): Promise<boolean> {
    const charter = this.project?.charter;
    if (!charter) return false;
    this.saving = true;
    this.editMessage = undefined;
    try {
      const next = {
        ...charter,
        title: draft.title,
        description: draft.description,
        location_label: draft.location,
      };
      await updateCharter(this.id, next);
      await this.load();
      return true;
    } catch (e) {
      this.editMessage = explain(e);
      return false;
    } finally {
      this.saving = false;
    }
  }
}
