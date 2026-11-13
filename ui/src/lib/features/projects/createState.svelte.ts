import {
  type Charter,
  type LocationChoice,
  type CharterMode,
  LOCATION_LABEL_MAX,
} from '#lib/domain/projects.ts';
import {
  emptyPlaceValue,
  fitLabel,
  intentPlace,
  type Place,
  type PlaceValue,
} from '#lib/domain/places.ts';
import { tagScope } from '#lib/domain/scopes.ts';
import { explain } from '#lib/services/errors.ts';
import { createProject, projectAllowance } from '#lib/services/projects.ts';

import type { EventStepId } from '#lib/domain/eventCreate.ts';

type Step = { id: EventStepId; title: string };

export const CREATE_STEPS: Step[] = [
  { id: 'basics', title: 'Basics' },
  { id: 'location', title: 'Location' },
  { id: 'scope', title: 'Scope' },
  { id: 'overview', title: 'Overview' },
];

/** The new-proposal wizard's draft (web's CreateProjectPage state without its Type step: a
 * proposal has no type until its leading plan gives it one, ADR 0059); `EventCreateState`
 * extends it for private events. */
export class CreateState {
  step = $state(0);
  mode = $state<CharterMode>('proposal');
  readonly steps: Step[] = $derived(CREATE_STEPS);
  title = $state('');
  description = $state('');
  intent = $state<LocationChoice>('later');
  /** The Location step's picker (web's `locationValue`). */
  placeValue = $state<PlaceValue>(emptyPlaceValue());
  tags = $state<string[]>([]);
  submitting = $state(false);
  message = $state<string>();
  /** Projects left this week for a New member; `null`: no limit. */
  allowance = $state<number | null>(null);

  readonly stepId = $derived(this.steps[this.step]?.id ?? 'basics');
  readonly locationLabel = $derived(
    this.intent === 'online'
      ? 'Online'
      : this.intent === 'physical'
        ? this.place.trim() || 'Physical'
        : 'Decide later',
  );
  readonly missing = $derived.by(() => {
    if (this.stepId === 'basics') {
      return [!this.title.trim() && 'Title', !this.description.trim() && 'Description'].filter(
        (m): m is string => !!m,
      );
    }
    if (this.stepId === 'location' && this.intent === 'physical' && !this.place.trim())
      return ['Location'];
    if (this.stepId === 'scope' && this.scopeNeeded()) return [this.scopeNeeded()];
    return [];
  });

  /** web: a project needs a channel tag (events override it); '' when nothing is needed. */
  protected scopeNeeded(): string {
    return this.tags.some((t) => tagScope(t)?.kind === 'channel') ? '' : 'Channel';
  }

  /** The picked place's label (cut to the records' limit, as the old field's maxlength was). */
  get place(): string {
    return this.placeValue.displayLabel;
  }

  setPlace(value: PlaceValue): void {
    this.placeValue = { ...value, displayLabel: fitLabel(value.displayLabel, LOCATION_LABEL_MAX) };
  }

  /** The record's place for `label`: online, or picked with coordinates (ADR 0078). */
  placeFor(label: string): Place | null {
    return intentPlace(this.intent, this.placeValue, label);
  }

  constructor(mode: CharterMode = 'proposal') {
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
    const label = this.intent === 'later' ? '' : this.locationLabel;
    return {
      title: this.title.trim(),
      description: this.description.trim(),
      mode: this.mode,
      subtype: 'standard',
      location_label: label,
      place: this.placeFor(label),
      phase: 'Proposal',
      tags: this.tags,
    };
  }

  /** The new project's id, or undefined with `message` set. */
  async submit(): Promise<string | undefined> {
    this.submitting = true;
    this.message = undefined;
    try {
      const id = await createProject(this.charter());
      return id;
    } catch (e) {
      this.message = explain(e);
      return undefined;
    } finally {
      this.submitting = false;
    }
  }
}
