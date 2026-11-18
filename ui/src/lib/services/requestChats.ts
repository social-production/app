import type { Activity } from '#lib/domain/activities.ts';
import { plannersSeed, type ServiceRequest } from '#lib/domain/serviceRequests.ts';

import { addGroupMember, chatSummary, removeGroupMember, startChat } from './messages.ts';
import { myAgent } from './people.ts';

/**
 * A collective request's planners' chat (ADR 0113): the member who plans a request opens it
 * with the person asking; whoever takes a role on the planned activity joins it, and leaves it
 * with the role. Only a group's creator manages its people, so the planner's device keeps it in
 * step each time it loads the service.
 */

/** The details the chat opens with. */
function opening(r: ServiceRequest): string {
  return r.body.trim() ? `${r.title}\n\n${r.body.trim()}` : r.title;
}

/** Opens the chat once its activity is written (does nothing if it is already there). */
export async function openPlannersChat(r: ServiceRequest): Promise<string> {
  const me = await myAgent();
  const seed = plannersSeed(me, r.id);
  const there = await chatSummary(seed, 0).then(
    () => true,
    () => false,
  );
  if (!there) await startChat('group', [r.requester], r.title.slice(0, 60), opening(r), seed);
  return seed;
}

/** Adds the activity's role-takers and the person asking, and removes anyone who left a role. */
export async function syncPlannersChat(r: ServiceRequest, a: Activity): Promise<void> {
  const me = await myAgent();
  if (a.author !== me) return;
  const seed = plannersSeed(me, r.id);
  const summary = await chatSummary(seed, 0).catch(() => null);
  if (!summary) {
    await openPlannersChat(r);
    return syncPlannersChat(r, a);
  }
  const want = new Set([r.requester, ...a.roles.flatMap((role) => role.people)]);
  want.delete(me);
  const have = new Set(summary.members.filter((m) => m !== me));
  for (const agent of want) if (!have.has(agent)) await addGroupMember(seed, agent);
  for (const agent of have) if (!want.has(agent)) await removeGroupMember(seed, agent);
}

/** Whether you are in a planned request's chat (its planner, the person asking or a role). */
export function inPlannersChat(me: string, r: ServiceRequest, a: Activity): boolean {
  return me === a.author || me === r.requester || a.roles.some((role) => role.people.includes(me));
}
