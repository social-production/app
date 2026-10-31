import type { Activity } from '#lib/domain/activities.ts';
import { HELP_MODE, roleCounts, type HelpRoleDraft } from '#lib/domain/helpRequests.ts';
import type { Contact } from '#lib/domain/messages.ts';
import { shareMessage } from '#lib/domain/share.ts';

import { createActivity, listActivities, setCommitment } from './activities.ts';
import { sendNewChat } from './messages.ts';
import { createProject, joinProject } from './projects.ts';

/** What web's create-help-request page sends. */
export interface HelpDraft {
  title: string;
  body: string;
  online: boolean;
  /** The place, or "Online". */
  location: string;
  /** Holochain microseconds. */
  startsAt: number;
  endsAt: number;
  tags: string[];
  roles: HelpRoleDraft[];
}

/**
 * Found the request's space in the commons and list it, then write its one activity: the
 * time, place and roles (ADR 0067). The activity may wait a moment for the new listing.
 */
export async function createHelpRequest(d: HelpDraft): Promise<string> {
  const id = await createProject({
    title: d.title,
    description: d.body,
    mode: HELP_MODE,
    subtype: 'standard',
    location_label: d.location,
    phase: 'Help request',
    tags: d.tags,
  });
  const activity = {
    title: d.title,
    startsAt: d.startsAt,
    endsAt: d.endsAt,
    online: d.online,
    place: d.location,
    meetingUrl: null,
    note: d.body,
    planStage: null,
    roles: d.roles.map((r) => ({
      label: r.label,
      description: r.description,
      suggest: null,
      ...roleCounts(r.slots),
    })),
  };
  for (let attempt = 1; ; attempt++) {
    try {
      await createActivity(id, activity);
      return id;
    } catch (e) {
      if (attempt >= 5) throw e;
      await new Promise((resolve) => setTimeout(resolve, 1500));
    }
  }
}

/** The request's one activity (its time, place and roles), once it has reached this node. */
export async function helpActivity(id: string): Promise<Activity | null> {
  return (await listActivities(id))[0] ?? null;
}

/** web's commit/uncommit: committing joins the request first (ADR 0067). */
export async function commitToRole(
  id: string,
  activity: string,
  role: number | null,
  member: boolean,
): Promise<void> {
  if (role !== null && !member) await joinProject(id, true);
  await setCommitment(id, activity, role);
}

/**
 * shares.py `share_help_request_with_user`: the link goes into your direct chat with them
 * (started if you have none), and their device shows it as an `hr-share` notification.
 */
export async function shareHelpRequest(to: Contact, title: string, url: string): Promise<void> {
  await sendNewChat({
    kind: 'direct',
    members: [to],
    title: '',
    body: shareMessage('help request', title, url),
  });
}
