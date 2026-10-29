import {
  PROJECT_TYPES,
  type Charter,
  type LocationChoice,
  type CharterMode,
} from '#lib/domain/projects.ts';
import { tagScope } from '#lib/domain/scopes.ts';
import { explain } from '#lib/services/errors.ts';
import { createProject, projectAllowance } from '#lib/services/projects.ts';

import type { EventStepId } from '#lib/domain/eventCreate.ts';

type Step = { id: 'type' | EventStepId; title: string };

export const CREATE_STEPS: Step[] = [
  { id: 'type', title: 'Type' },
  { id: 'basics', title: 'Basics' },
  { id: 'location', title: 'Location' },
  { id: 'scope', title: 'Scope' },
  { id: 'overview', title: 'Overview' },
];

/** The create-project wizard's draft (web's CreateProjectPage state); `EventCreateState`
 * extends it for events. */
export class CreateState {
  step = $state(0);
  mode = $state<CharterMode>('productive');
  readonly steps: Step[] = $derived(CREATE_STEPS);
  title = $state('');
  description = $state('');
  intent = $state<LocationChoice>('later');
  place = $state('');
  tags = $state<string[]>([]);
  submitting = $state(false);
  message = $state<string>();
  /** Projects left this week for a New member; `null`: no limit. */
  allowance = $state<number | null>(null);

  readonly stepId = $derived(this.steps[this.step]?.id ?? 'basics');
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
    if (this.stepId === 'scope' && this.scopeNeeded()) return [this.scopeNeeded()];
    return [];
  });

  /** web: a project needs a channel tag (events override it); '' when nothing is needed. */
  protected scopeNeeded(): string {
    return this.tags.some((t) => tagScope(t)?.kind === 'channel') ? '' : 'a channel tag';
  }

  constructor(mode: CharterMode = 'productive') {
    this.mode = mode;
  }

  /** The overview's link back to a step. */
  stepOf(id: Step['id']): number {
    return Math.max(
      this.steps.findIndex((s) => s.id === id),
      0,
    );
  }

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
