import { t } from '#lib/i18n/index.ts';
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
import {
  createProject,
  getProject,
  linkLineage,
  projectAllowance,
} from '#lib/services/projects.ts';

import type { EventStepId } from '#lib/domain/eventCreate.ts';

import { ScopeArt } from './scopes/scopeArt.svelte.ts';

type Step = { id: EventStepId; title: string; question?: string };

export const CREATE_STEPS: Step[] = [
  { id: 'basics', title: t('Basics'), question: t('What do you propose?') },
  { id: 'scope', title: t('Tags'), question: t('Where should people find it?') },
  { id: 'location', title: t('Location'), question: t('Where does it happen?') },
  { id: 'overview', title: t('Overview'), question: t('Check and publish') },
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
  /** Convert (ADR 0106): the converted project this proposal continues. */
  continues = $state<string | null>(null);
  submitting = $state(false);
  message = $state<string>();
  /** Projects left this week for a New member; `null`: no limit. */
  allowance = $state<number | null>(null);
  /** A picture and cover from the start (ADR 0117), optional: the type's tile stays if none. */
  readonly art = new ScopeArt();

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
    return this.tags.some((t) => tagScope(t)?.kind === 'channel') ? '' : t('Channel');
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
      picture: this.art.newPicture?.hash ?? null,
      cover: this.art.newCover?.hash ?? null,
    };
  }

  /** Convert (ADR 0106): start from the converted project's title, description and tags. */
  async continueFrom(id: string): Promise<void> {
    this.continues = id;
    try {
      const charter = (await getProject(id)).charter;
      if (!charter) return;
      this.title = charter.title;
      this.description = charter.description;
      this.tags = [...charter.tags];
      if (charter.location_label.trim().toLowerCase() === 'online') this.intent = 'online';
    } catch (e) {
      this.message = explain(e);
    }
  }

  /** The new project's id, or undefined with `message` set. */
  async submit(): Promise<string | undefined> {
    this.submitting = true;
    this.message = undefined;
    try {
      const id = await createProject(this.charter());
      if (this.continues) {
        // Each names the other; a failure here leaves the new proposal standing.
        await linkLineage(id, this.continues, false).catch(() => undefined);
        await linkLineage(this.continues, id, true).catch(() => undefined);
      }
      return id;
    } catch (e) {
      this.message = explain(e);
      return undefined;
    } finally {
      this.submitting = false;
    }
  }
}
