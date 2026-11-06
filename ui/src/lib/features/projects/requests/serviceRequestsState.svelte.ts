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
}

const emptyForm = (): RequestForm => ({ title: '', body: '', scheduledAt: '', endsAt: '' });

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
  /** The request whose decline reason is being written (web's `decliningRequestId`). */
  declining = $state('');
  declineReason = $state('');

  constructor(
    private readonly project: string,
    private readonly me: () => string,
    private readonly reviewer: () => boolean,
  ) {}

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

  addPreferredTime(): void {
    this.form = { ...this.form, ...preferredTime() };
  }

  async send(service: {
    title: string;
    creator: string;
    settings: RequestSettings;
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
    const refused = refusal(service.settings, !!f.scheduledAt);
    if (refused) {
      this.error = refused;
      return false;
    }
    return this.run(async () => {
      await sendServiceRequest(
        { id: this.project, ...service },
        {
          title: f.title,
          body: f.body,
          startsAt: micros(f.scheduledAt),
          endsAt: micros(f.endsAt),
        },
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
    });
  }

  openSettings(current: RequestSettings): void {
    this.settingsForm = { ...current };
    this.error = '';
    this.settingsOpen = true;
  }

  async saveSettings(): Promise<boolean> {
    const s = this.settingsForm;
    return this.run(async () => {
      await saveRequestSettings(this.project, s);
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
