import {
  PROJECT_TYPES,
  type Charter,
  type LocationChoice,
  type ProjectMode,
} from '#lib/domain/projects.ts';
import { explain } from '#lib/services/errors.ts';
import { createProject, projectAllowance } from '#lib/services/projects.ts';

export const CREATE_STEPS = [
  { id: 'type', title: 'Type' },
  { id: 'basics', title: 'Basics' },
  { id: 'location', title: 'Location' },
  { id: 'scope', title: 'Scope' },
  { id: 'overview', title: 'Overview' },
] as const;

/** The create-project wizard's draft (web's CreateProjectPage state). */
export class CreateState {
  step = $state(0);
  mode = $state<ProjectMode>('productive');
  title = $state('');
  description = $state('');
  intent = $state<LocationChoice>('later');
  place = $state('');
  tags = $state<string[]>([]);
  submitting = $state(false);
  message = $state<string>();
  /** Projects left this week for a New member; `null`: no limit. */
  allowance = $state<number | null>(null);

  readonly stepId = $derived(CREATE_STEPS[this.step]?.id ?? 'type');
  readonly typeLabel = $derived(PROJECT_TYPES.find((t) => t.value === this.mode)?.label ?? '');
  readonly locationLabel = $derived(
    this.intent === 'online'
      ? 'Online'
      : this.intent === 'physical'
        ? this.place.trim() || 'Physical'
        : 'Decide later',
  );
  readonly missing = $derived.by(() => {
    if (this.stepId === 'basics') {
      return [!this.title.trim() && 'title', !this.description.trim() && 'description'].filter(
        (m): m is string => !!m,
      );
    }
    if (this.stepId === 'location' && this.intent === 'physical' && !this.place.trim())
      return ['place'];
    return [];
  });

  async checkAllowance(): Promise<void> {
    try {
      this.allowance = await projectAllowance();
    } catch {
      this.allowance = null; // the zome still enforces the limit on submit
    }
  }

  charter(): Charter {
    return {
      title: this.title.trim(),
      description: this.description.trim(),
      mode: this.mode,
      subtype: 'standard',
      location_label: this.intent === 'later' ? '' : this.locationLabel,
      phase: 'Proposal',
      tags: this.tags,
    };
  }

  /** The new project's id, or undefined with `message` set. */
  async submit(): Promise<string | undefined> {
    this.submitting = true;
    this.message = undefined;
    try {
      return await createProject(this.charter());
    } catch (e) {
      this.message = explain(e);
      return undefined;
    } finally {
      this.submitting = false;
    }
  }
}
