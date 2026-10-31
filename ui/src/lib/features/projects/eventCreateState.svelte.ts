import {
  eventSteps,
  ORGANIZER_DEMAND_NOTE,
  upfrontPlanProblems,
  type EventStepId,
} from '#lib/domain/eventCreate.ts';
import type { Contact } from '#lib/domain/messages.ts';
import { emptyPlanForm, planFromForm, type PlanForm } from '#lib/domain/planForm.ts';
import type { EventAudience, EventGovernance } from '#lib/domain/projects.ts';
import type { Scope } from '#lib/domain/scopes.ts';
import { myClosedCommunities } from '#lib/services/closedCommunities.ts';
import { explain } from '#lib/services/errors.ts';
import { createEvent } from '#lib/services/events.ts';
import { contacts } from '#lib/services/messages.ts';
import { myAgent } from '#lib/services/people.ts';

import { CreateState } from './createState.svelte.ts';

/** web's CreateEventPage draft: the project draft plus audience, people, control and plan. */
export class EventCreateState extends CreateState {
  audience = $state<EventAudience>('public');
  /** web preselects organizer-controlled for private events. */
  chosenGovernance = $state<EventGovernance>('organizer_controlled');
  invitees = $state<Contact[]>([]);
  organizers = $state<Contact[]>([]);
  /** web's home community: the one private community a `private_community` event is for. */
  community = $state<Scope | null>(null);
  plan = $state<PlanForm>(emptyPlanForm('standard'));
  private people: Promise<Contact[]> | undefined;
  private communities: Promise<Scope[]> | undefined;
  private seenCommunities: Scope[] = [];

  readonly isPrivate = $derived(this.audience !== 'public');
  readonly governance = $derived<EventGovernance>(
    this.isPrivate ? this.chosenGovernance : 'collaborative',
  );
  readonly upfrontPlan = $derived(this.governance === 'organizer_controlled');
  override readonly steps = $derived(eventSteps(this.isPrivate, this.upfrontPlan));
  readonly planProblems = $derived(
    this.upfrontPlan ? upfrontPlanProblems(this.planWithPlace(), this.intent === 'physical') : [],
  );
  readonly canContinue = $derived(this.stepReady(this.stepId));

  constructor() {
    super('event');
  }

  /** web's per-step gates (basics and location come from the project draft's `missing`). */
  private stepReady(id: EventStepId | 'type'): boolean {
    if (id === 'scope' && this.audience === 'private_community') return !!this.community;
    if (id === 'scope') return this.audience !== 'invite_only' || this.invitees.length > 0;
    if (id === 'plan') return this.planProblems.length === 0;
    return true;
  }

  /** web copies the Location step's place into the plan ("TBD" when deciding later, as
   * web-backend's `create_event` does); event plans need a place. */
  private planWithPlace(): PlanForm {
    const location =
      this.intent === 'online' ? 'Online' : this.intent === 'physical' ? this.place.trim() : 'TBD';
    const stages = this.plan.stages.filter((s) => s.title.trim() || s.details.trim());
    return { ...this.plan, stages, location, demandNote: ORGANIZER_DEMAND_NOTE };
  }

  /** web: public events need at least one channel or community tag; private ones none. */
  protected override scopeNeeded(): string {
    return this.isPrivate || this.tags.length > 0 ? '' : 'a channel or community tag';
  }

  /** People matching `query` by name, not yourself (as the compose sheet searches). */
  async search(query: string): Promise<Contact[]> {
    this.people ??= Promise.all([contacts(), myAgent()]).then(([all, me]) =>
      all.filter((c) => c.agent !== me),
    );
    const needle = query.trim().toLowerCase();
    try {
      return (await this.people).filter((c) => c.name.toLowerCase().includes(needle)).slice(0, 6);
    } catch {
      this.people = undefined;
      return [];
    }
  }

  /** Your private communities whose handle contains `query` (web lists only private ones for
   * this audience), as picker rows: the seed and web's "<name> (Private)" label. */
  async searchCommunities(query: string): Promise<Contact[]> {
    this.communities ??= myClosedCommunities();
    const needle = query.trim().toLowerCase();
    try {
      this.seenCommunities = await this.communities;
      return this.seenCommunities
        .filter((c) => c.slug.includes(needle) || c.handle.toLowerCase().includes(needle))
        .slice(0, 6)
        .map((c) => ({ agent: c.id, name: `${c.handle} (Private)` }));
    } catch {
      this.communities = undefined;
      return [];
    }
  }

  /** web's addCommunityTag for this audience: exactly one community. */
  chooseCommunity(seed: string | null): void {
    this.community = this.seenCommunities.find((c) => c.id === seed) ?? null;
  }

  addInvitee(c: Contact): void {
    if (!this.invitees.some((i) => i.agent === c.agent)) this.invitees = [...this.invitees, c];
  }

  /** web: an organizer of an invite-only event is invited too. */
  addOrganizer(c: Contact): void {
    if (!this.organizers.some((o) => o.agent === c.agent))
      this.organizers = [...this.organizers, c];
    if (this.audience === 'invite_only') this.addInvitee(c);
  }

  /** The new event's page id (listing id, or a private event's seed), or undefined with
   * `message` set. */
  override async submit(): Promise<string | undefined> {
    if (this.planProblems[0]) {
      this.message = this.planProblems[0];
      return undefined;
    }
    this.submitting = true;
    this.message = undefined;
    try {
      return await createEvent({
        charter: this.charter(),
        audience: this.audience,
        organized: this.isPrivate && this.upfrontPlan,
        community: this.audience === 'private_community' ? this.community : null,
        invitees: this.audience === 'invite_only' ? this.invitees.map((c) => c.agent) : [],
        organizers: this.isPrivate ? this.organizers.map((c) => c.agent) : [],
        plan: this.upfrontPlan ? planFromForm('Event', this.planWithPlace()) : null,
      });
    } catch (e) {
      this.message = explain(e);
      return undefined;
    } finally {
      this.submitting = false;
    }
  }
}
