import type { EventSchedule } from '#lib/domain/eventSchedule.ts';
import type { Plan } from '#lib/domain/plans.ts';
import type { InviteKind } from '#lib/domain/notifications.ts';
import { isPrivateSeed, type Charter } from '#lib/domain/projects.ts';
import {
  callCell,
  callZome,
  newPrivateEventSeed,
  projectCell,
  stringToHash,
  type ActionHash,
} from '#lib/holo/index.ts';

import { propose } from './governance.ts';
import { inboxInvitations } from './messages.ts';
import { myAgent, namesOf } from './people.ts';
import { listPlans, submitPlan } from './plans.ts';
import { createProject, getProject, joinProject } from './projects.ts';

/** web's create-event form, once its steps are filled in (ADR 0057, 0061). */
export interface EventDraft {
  charter: Charter;
  audience: 'public' | 'invite_only';
  organized: boolean;
  /** People to invite and co-organizers (agent keys); co-organizers are invited too. */
  invitees: string[];
  organizers: string[];
  /** The plan an organizer-controlled event starts with. */
  plan: Plan | null;
}

const READY = 'The organizers are ready.';

/** Tell `to` about a private event through their inbox (ADR 0061, as chats do, ADR 0043). */
export async function sendEventInvite(to: string, seed: string): Promise<void> {
  await callZome<null>('conversation', 'messages', 'send_invite', { to: stringToHash(to), seed });
}

/**
 * Publish an event. A public one is listed like a project; a private one isn't: its seed is
 * its address, its organizers are granted, everyone (yourself too, for your other devices)
 * is invited, and an organizer-controlled one starts in Activity with its plan.
 */
export async function createEvent(d: EventDraft): Promise<string> {
  if (d.audience === 'public') return createProject(d.charter);
  const seed = newPrivateEventSeed(await myAgent(), d.organized);
  const cell = await projectCell(seed, d.charter.title);
  await callCell<ActionHash>(cell, 'project', 'create_charter', d.charter);
  for (const agent of d.organizers) {
    await callCell<ActionHash>(cell, 'project', 'add_organizer', stringToHash(agent));
  }
  if (d.organized && d.plan) {
    await propose(seed, { Phase: { to: 'phase-2', plan: null, reason: READY } });
    await submitPlan(seed, d.plan);
    await propose(seed, { Phase: { to: 'phase-5', plan: null, reason: READY } });
  }
  const everyone = new Set([await myAgent(), ...d.organizers, ...d.invitees]);
  for (const agent of everyone) await sendEventInvite(agent, seed);
  return seed;
}

/** Make someone a co-organizer of a private event and invite them. */
export async function addOrganizer(seed: string, agent: string): Promise<void> {
  const cell = await projectCell(seed, 'Event');
  await callCell<ActionHash>(cell, 'project', 'add_organizer', stringToHash(agent));
  await sendEventInvite(agent, seed);
}

/** Private events in your inbox from other people, newest first. */
export async function eventInvitations(): Promise<{ seed: string; from: string; at: number }[]> {
  const me = await myAgent();
  return (await inboxInvitations())
    .filter((i) => isPrivateSeed(i.seed) && i.from !== me)
    .sort((a, b) => b.at - a.at);
}

/** web's PendingInvite for an event or a private community: waiting until you accept (join)
 * or decline. */
export interface PendingInvite {
  seed: string;
  kind: InviteKind;
  title: string;
  description: string;
  /** The inviter's key, and their name once found. */
  from: string;
  invitedBy: string | null;
  /** Holochain micros. */
  at: number;
  /** The leading approved plan's schedule, once there is one. */
  schedule: EventSchedule | null;
}

const declinedKey = (me: string) => `sp.invites.declined.${me}`;

/** Declines are kept on this device until they can be private entries (ADR 0061). */
function declined(me: string): string[] {
  try {
    const raw = localStorage.getItem(declinedKey(me));
    return raw ? (JSON.parse(raw) as string[]) : [];
  } catch {
    return [];
  }
}

/** Invites you haven't answered: reading each one joins its network, not the event or
 * community. */
export async function pendingInvites(): Promise<PendingInvite[]> {
  const me = await myAgent();
  const skip = new Set(declined(me));
  const open = (await eventInvitations()).filter((i) => !skip.has(i.seed));
  const found = await Promise.allSettled(
    open.map(async (i): Promise<PendingInvite | null> => {
      const p = await getProject(i.seed);
      if (p.amMember || !p.charter) return null;
      const kind = p.charter.mode === 'community' ? 'community' : 'event';
      const board = kind === 'event' ? await listPlans({ seed: i.seed }).catch(() => null) : null;
      const leading = board?.plans.find((v) => v.approved && v.leader === 'Leading');
      return {
        seed: i.seed,
        kind,
        title: p.charter.title,
        description: p.charter.description,
        from: i.from,
        invitedBy: null,
        at: i.at,
        schedule: leading?.plan.schedule ?? null,
      };
    }),
  );
  const invites = found.flatMap((r) => (r.status === 'fulfilled' && r.value ? [r.value] : []));
  const names = await namesOf([...new Set(invites.map((i) => i.from))]);
  return invites.map((i) => ({ ...i, invitedBy: names[i.from] ?? null }));
}

/** web's answerInvite: accepting joins the event or community; declining hides the invite on this device. */
export async function answerInvite(seed: string, accept: boolean): Promise<void> {
  if (accept) {
    // `join` renews a membership from before a removal, so a removed person can come back.
    await joinProject(seed, true);
    return;
  }
  const me = await myAgent();
  try {
    localStorage.setItem(declinedKey(me), JSON.stringify([...declined(me), seed]));
  } catch {
    // Private windows: the invite shows again next time.
  }
}
