import { fitsOpenSlot, type Slot } from '#lib/domain/availability.ts';
import { toLocalInput } from '#lib/domain/helpRequests.ts';
import type { RequestMode, RequestSettings } from '#lib/domain/projects.ts';
import {
  preferredTime,
  refusal,
  type RequestStatus,
  type ServiceRequest,
} from '#lib/domain/serviceRequests.ts';
import { explain } from '#lib/services/errors.ts';
import { namesOf } from '#lib/services/people.ts';
import {
  decideServiceRequest,
  myServiceRequests,
  saveRequestSettings,
  sendServiceRequest,
} from '#lib/services/serviceRequests.ts';

/** web's request composer (IndividualServicePhaseOne's `serviceRequestForm`). */
export interface RequestForm {
  title: string;
  body: string;
  scheduledAt: string;
  endsAt: string;
  /** The listed slot it starts from (web's `selectedActivityId`): its times are locked. */
  slot: string;
}

const emptyForm = (): RequestForm => ({
  title: '',
  body: '',
  scheduledAt: '',
  endsAt: '',
  slot: '',
});

const local = (micros: number): string => toLocalInput(new Date(micros / 1000));

const micros = (local: string): number | null => (local ? Date.parse(local) * 1000 : null);

/**
 * A personal service's requests (ADR 0076): the creator reviews every request sent to it;
 * anyone else sees their own. Each lives in the direct chat between the two people.
 */
export class ServiceRequestsState {
  requests = $state<ServiceRequest[]>([]);
  names = $state<Record<string, string>>({});
  loaded = $state(false);
  busy = $state(false);
  error = $state('');
  /** Which sheet is open. */
  listOpen = $state(false);
  composing = $state(false);
  settingsOpen = $state(false);
  form = $state<RequestForm>(emptyForm());
  settingsForm = $state<RequestSettings>({ enabled: true, mode: 'both', offSchedule: true });
  /** A collective service's settings change is voted on, with web's required reason. */
  settingsReason = $state('');
  /** The request whose decline reason is being written (web's `decliningRequestId`). */
  declining = $state('');
  declineReason = $state('');

  constructor(
    private readonly project: string,
    private readonly me: () => string,
    private readonly reviewer: () => boolean,
    /** An accepted timed request becomes a booking (ADR 0077). */
    private readonly book: (request: ServiceRequest) => Promise<void> = async () => {},
  ) {}

  /** Bookings already tried this session, so a failed one isn't retried on every load. */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- bookkeeping, never rendered
  private booked = new Set<string>();

  /** The requests this person may see, newest first. */
  get shown(): ServiceRequest[] {
    const me = this.me();
    return this.requests.filter((r) => (this.reviewer() ? r.requester !== me : r.requester === me));
  }

  async load(): Promise<void> {
    if (!this.me()) {
      this.loaded = true;
      return;
    }
    try {
      this.requests = await myServiceRequests(this.project);
      const agents = this.requests
        .map((r) => r.requester)
        .filter((a, i, all) => all.indexOf(a) === i);
      this.names = { ...this.names, ...(await namesOf(agents)) };
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.loaded = true;
    }
  }

  /** web's `openDockedServiceRequest`: a calendar-only service asks for a time at once. */
  openComposer(mode: RequestMode): void {
    this.form = { ...emptyForm(), ...(mode === 'calendar' ? preferredTime() : {}) };
    this.error = '';
    this.composing = true;
  }

  /** web's `prefillRequestFromSlot`: the slot's times, locked. */
  openForSlot(slot: Slot): void {
    this.form = {
      ...emptyForm(),
      scheduledAt: local(slot.startsAt),
      endsAt: local(slot.endsAt),
      slot: slot.id,
    };
    this.error = '';
    this.composing = true;
  }

  /** web's `openRequestForDay` without an open slot: that day 18:00–19:00. */
  openForDay(iso: string): void {
    this.form = { ...emptyForm(), scheduledAt: `${iso}T18:00`, endsAt: `${iso}T19:00` };
    this.error = '';
    this.composing = true;
  }

  addPreferredTime(): void {
    this.form = { ...this.form, ...preferredTime() };
  }

  /** `members`: a collective service's, who share its request chat (ADR 0089). */
  async send(service: {
    title: string;
    creator: string;
    settings: RequestSettings;
    slots: Slot[];
    members?: string[];
  }): Promise<boolean> {
    const f = this.form;
    if (!f.title.trim() || !f.body.trim()) {
      this.error = 'Add a title and details before sending the request.';
      return false;
    }
    if (!!f.scheduledAt !== !!f.endsAt) {
      this.error = 'Add both a start and a finish time, or neither.';
      return false;
    }
    const start = micros(f.scheduledAt);
    const end = micros(f.endsAt);
    // A collective service lists no slots: its times are its activities' (web's calendar).
    const fits =
      !!service.members ||
      start === null ||
      end === null ||
      fitsOpenSlot(service.slots, start, end);
    const refused = refusal(service.settings, !!f.scheduledAt, fits);
    if (refused) {
      this.error = refused;
      return false;
    }
    return this.run(async () => {
      await sendServiceRequest(
        {
          id: this.project,
          title: service.title,
          creator: service.creator,
          members: service.members,
        },
        { title: f.title, body: f.body, startsAt: start, endsAt: end },
      );
      this.composing = false;
      this.form = emptyForm();
    });
  }

  async decide(
    request: ServiceRequest,
    status: Exclude<RequestStatus, 'open'>,
    reason = '',
  ): Promise<boolean> {
    return this.run(async () => {
      await decideServiceRequest(request, status, reason);
      this.declining = '';
      this.declineReason = '';
      if (status === 'accepted') await this.bookOnce(request);
    });
  }

  /** web books an accepted timed request at once; a booking that didn't happen then (the
   * project call failed after the chat's) is made the next time the creator looks. */
  async heal(bookedRequests: string[]): Promise<void> {
    if (!this.reviewer()) return;
    try {
      for (const r of this.shown) {
        if (r.status === 'accepted' && !bookedRequests.includes(r.id)) await this.bookOnce(r);
      }
    } catch (e) {
      this.error = explain(e);
    }
  }

  private async bookOnce(request: ServiceRequest): Promise<void> {
    if (request.startsAt === null || request.endsAt === null || this.booked.has(request.id)) return;
    this.booked.add(request.id);
    await this.book(request);
  }

  /** A collective request whose activity was written but whose acceptance wasn't (the chat
   * call failed after the project's) is accepted the next time a member looks (ADR 0089). */
  async healPlanned(bookedRequests: string[]): Promise<void> {
    if (!this.reviewer()) return;
    for (const r of this.shown) {
      if (r.status !== 'open' || !bookedRequests.includes(r.id) || this.booked.has(r.id)) continue;
      this.booked.add(r.id);
      await decideServiceRequest(r, 'accepted').catch((e: unknown) => (this.error = explain(e)));
    }
  }

  openSettings(current: RequestSettings): void {
    this.settingsReason = '';
    this.settingsForm = { ...current };
    this.error = '';
    this.settingsOpen = true;
  }

  async saveSettings(): Promise<boolean> {
    const s = this.settingsForm;
    return this.run(async () => {
      await saveRequestSettings(this.project, s, this.settingsReason);
      this.settingsOpen = false;
    });
  }

  private async run(work: () => Promise<void>): Promise<boolean> {
    this.busy = true;
    this.error = '';
    try {
      await work();
      await this.load();
      return true;
    } catch (e) {
      this.error = explain(e);
      return false;
    } finally {
      this.busy = false;
    }
  }
}
