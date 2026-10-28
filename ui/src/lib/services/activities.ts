import type { Activity } from '#lib/domain/activities.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { cellFor, type ProjectRef } from './projects.ts';

interface SuggestionWire {
  id: ActionHash;
  person: AgentPubKey;
  by: AgentPubKey;
  at: number;
}

const toSuggestion = (s: SuggestionWire) => ({
  id: hashToString(s.id),
  person: hashToString(s.person),
  by: hashToString(s.by),
  at: s.at,
});

/** The activities zome's `ActivityView` (ADR 0056). */
interface ActivityWire {
  id: ActionHash;
  author: AgentPubKey;
  created_at: number;
  title: string;
  starts_at: number;
  ends_at: number;
  online: boolean;
  place: string;
  meeting_url: string | null;
  note: string;
  plan_stage: string | null;
  roles: {
    label: string;
    required: number;
    maximum: number | null;
    description?: string;
    people: AgentPubKey[];
    taken_at: number[];
    suggestion: SuggestionWire | null;
    declined: SuggestionWire[];
  }[];
  my_role: number | null;
  roles_open: boolean;
  ratings: { author: AgentPubKey; stars: number; comment: string; at: number }[];
  can_rate: boolean;
  completion: {
    eligible: number;
    completed: number;
    uncompleted: number;
    mine: boolean | null;
    can_set: boolean;
  };
}

export interface NewActivityRole {
  label: string;
  required: number;
  maximum: number | null;
  /** web's suggested person for the role, by agent key. */
  suggest: string | null;
  /** A help request's role description (ADR 0067). */
  description?: string;
}

export interface NewActivity {
  title: string;
  /** Holochain microseconds. */
  startsAt: number;
  endsAt: number;
  online: boolean;
  place: string;
  meetingUrl: string | null;
  note: string;
  planStage: string | null;
  roles: NewActivityRole[];
}

const activities = async <T>(project: ProjectRef, fn: string, payload?: unknown): Promise<T> =>
  callCell<T>(await cellFor(project), 'activities', fn, payload);

function toActivity(w: ActivityWire): Activity {
  return {
    id: hashToString(w.id),
    author: hashToString(w.author),
    createdAt: w.created_at,
    title: w.title,
    startsAt: w.starts_at,
    endsAt: w.ends_at,
    online: w.online,
    place: w.place,
    meetingUrl: w.meeting_url,
    note: w.note,
    planStage: w.plan_stage,
    roles: w.roles.map((r, index) => ({
      index,
      label: r.label,
      required: r.required,
      maximum: r.maximum,
      description: r.description ?? '',
      people: r.people.map(hashToString),
      takenAt: r.taken_at,
      suggestion: r.suggestion && toSuggestion(r.suggestion),
      // Older coordinators don't send it (a hosted node may lag a deploy).
      declined: (r.declined ?? []).map(toSuggestion),
    })),
    myRole: w.my_role,
    rolesOpen: w.roles_open,
    ratings: w.ratings.map((r) => ({ ...r, author: hashToString(r.author) })),
    canRate: w.can_rate,
    completion: {
      eligible: w.completion.eligible,
      completed: w.completion.completed,
      uncompleted: w.completion.uncompleted,
      mine: w.completion.mine,
      canSet: w.completion.can_set,
    },
  };
}

export async function listActivities(project: ProjectRef): Promise<Activity[]> {
  return (await activities<ActivityWire[]>(project, 'list_activities')).map(toActivity);
}

export async function createActivity(project: ProjectRef, a: NewActivity): Promise<string> {
  const id = await activities<ActionHash>(project, 'create_activity', {
    title: a.title,
    starts_at: a.startsAt,
    ends_at: a.endsAt,
    online: a.online,
    place: a.place,
    meeting_url: a.meetingUrl,
    note: a.note,
    plan_stage: a.planStage,
    roles: a.roles.map((r) => ({
      label: r.label,
      required: r.required,
      maximum: r.maximum,
      description: r.description ?? '',
      suggest: r.suggest ? stringToHash(r.suggest) : null,
    })),
  });
  return hashToString(id);
}

/** web's changecommitment: take `role`, or leave your role when `role` is null. */
export async function setCommitment(
  project: ProjectRef,
  activity: string,
  role: number | null,
): Promise<void> {
  if (role === null) {
    await activities<null>(project, 'leave_role', stringToHash(activity));
  } else {
    await activities<ActionHash>(project, 'take_role', { activity: stringToHash(activity), role });
  }
}

export async function suggestRole(
  project: ProjectRef,
  activity: string,
  role: number,
  person: string,
): Promise<void> {
  await activities<ActionHash>(project, 'suggest_role', {
    activity: stringToHash(activity),
    role,
    person: stringToHash(person),
  });
}

export async function declineSuggestion(project: ProjectRef, suggestion: string): Promise<void> {
  await activities<ActionHash>(project, 'decline_suggestion', stringToHash(suggestion));
}

/** Rate 1–5 with a comment, or clear your rating with `stars` null. */
export async function rateActivity(
  project: ProjectRef,
  activity: string,
  stars: number | null,
  comment: string,
): Promise<void> {
  await activities<ActionHash>(project, 'rate_activity', {
    activity: stringToHash(activity),
    stars,
    comment,
  });
}

/** web's history completion: it happened, it didn't, or clear your check-in. */
export async function checkIn(
  project: ProjectRef,
  activity: string,
  done: boolean | null,
): Promise<void> {
  await activities<ActionHash>(project, 'check_in', { activity: stringToHash(activity), done });
}
