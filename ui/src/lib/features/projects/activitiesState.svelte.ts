import type { Activity } from '#lib/domain/activities.ts';
import { defaultTimes } from '#lib/domain/activityCalendar.ts';
import { namedRoles, newActivityForm, type ActivityForm } from '#lib/domain/activityForm.ts';
import { rankMentions, type MentionPool } from '#lib/domain/mentions.ts';
import type { PlanBoard, PlanKind } from '#lib/domain/plans.ts';
import {
  checkIn,
  createActivity,
  declineSuggestion,
  listActivities,
  rateActivity,
  setCommitment,
  suggestRole,
} from '#lib/services/activities.ts';
import { explain } from '#lib/services/errors.ts';
import { mentionPool } from '#lib/services/messages.ts';
import { namesOf } from '#lib/services/people.ts';

/** A linked plan stage: web's `production:<stage>` id, its label and its plan's group. */
export interface StageOption {
  id: string;
  label: string;
  group: string;
}

/**
 * web's Activity phase (ProductiveLifecyclePhaseFive + ActivitySchedulingPanel): the project's
 * activities, the composer, and the writes behind take/leave, suggest and decline (ADR 0056).
 * One write at a time, as the chain takes them.
 */
export class ActivitiesState {
  activities = $state<Activity[]>([]);
  names = $state<Record<string, string>>({});
  loaded = $state(false);
  error = $state('');
  composing = $state(false);
  busy = $state(false);
  form = $state<ActivityForm>(newActivityForm());
  selectedDay = $state('');
  highlighted = $state<string | null>(null);
  private pool: Promise<MentionPool> | undefined;
  private queue: Promise<unknown> = Promise.resolve();

  constructor(
    readonly projectId: string,
    readonly me: () => string,
  ) {}

  readonly live = $derived(this.activities.filter((a) => a.rolesOpen));
  readonly past = $derived(this.activities.filter((a) => !a.rolesOpen));

  async load(): Promise<void> {
    try {
      const all = await listActivities(this.projectId);
      const people = all.flatMap((a) => [
        a.author,
        ...a.roles.flatMap((r) => [...r.people, ...(r.suggestion ? [r.suggestion.person] : [])]),
        ...a.ratings.map((r) => r.author),
      ]);
      const unique = people.filter((a, i) => people.indexOf(a) === i);
      this.names = { ...this.names, ...(await namesOf(unique)) };
      this.activities = all;
      this.error = '';
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.loaded = true;
    }
  }

  private write(task: () => Promise<unknown>): Promise<void> {
    const run = this.queue.then(task).then(
      () => this.load(),
      (e: unknown) => {
        this.error = explain(e);
      },
    );
    this.queue = run;
    return run;
  }

  openComposer(): void {
    this.form = {
      ...newActivityForm(),
      ...defaultTimes(this.selectedDay || null, this.activities),
    };
    this.error = '';
    this.composing = true;
  }

  async submit(): Promise<void> {
    const f = this.form;
    this.busy = true;
    try {
      await createActivity(this.projectId, {
        title: f.title.trim(),
        startsAt: Date.parse(f.scheduledAt) * 1000,
        endsAt: Date.parse(f.endsAt) * 1000,
        online: f.online,
        place: f.online ? f.place.trim() || 'Online' : f.place.trim(),
        meetingUrl: f.online && f.meetingUrl.trim() ? f.meetingUrl.trim() : null,
        note: f.note.trim(),
        planStage: f.planStage,
        roles: namedRoles(f.roles).map((r) => ({ ...r, suggest: r.suggest?.agent ?? null })),
      });
      this.composing = false;
      await this.load();
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.busy = false;
    }
  }

  commit = (activity: string, role: number | null): Promise<void> =>
    this.write(() => setCommitment(this.projectId, activity, role));

  suggest = (activity: string, role: number, agent: string): Promise<void> =>
    this.write(() => suggestRole(this.projectId, activity, role, agent));

  checkIn = (activity: string, done: boolean): Promise<void> =>
    this.write(() => checkIn(this.projectId, activity, done));

  rate = (activity: string, stars: number | null, comment: string): Promise<void> =>
    this.write(() => rateActivity(this.projectId, activity, stars, comment));

  decline = (suggestion: string): Promise<void> =>
    this.write(() => declineSuggestion(this.projectId, suggestion));

  /** web's searchPeopleSuggestions: people you follow, then your followers, then everyone. */
  searchPeople = async (q: string): Promise<{ agent: string; name: string }[]> => {
    this.pool ??= mentionPool();
    const pool = await this.pool;
    return rankMentions(q, pool.people, [pool.following, pool.followers, pool.partners], this.me());
  };
}

/** web's `selectablePlanPhases`: the stages of the leading approved plans, by plan kind. */
export function stageOptions(board: PlanBoard | undefined, mode: string): StageOption[] {
  const service = mode === 'collective-service';
  const kinds: { kind: PlanKind; key: string; label: string }[] = [
    service
      ? { kind: 'Organisation', key: 'production', label: 'Operations' }
      : { kind: 'Production', key: 'production', label: 'Production' },
    service
      ? { kind: 'Access', key: 'distribution', label: 'Access' }
      : { kind: 'Distribution', key: 'distribution', label: 'Distribution' },
  ];
  return kinds.flatMap(({ kind, key, label }) => {
    const plan = board?.plans.find(
      (p) => p.plan.kind === kind && p.leader === 'Leading' && p.approved,
    );
    return (plan?.plan.stages ?? []).map((s, i) => ({
      id: `${key}:${i}`,
      label: `${label}: ${s.title}`,
      group: label,
    }));
  });
}
