import { callCell, hashToString, stringToHash, type AgentPubKey } from '#lib/holo/index.ts';

import { cellOf } from './projects.ts';

/** One invitation recorded on an event (ADR 0111): who, by whom, when (ms). */
export interface EventInvite {
  agent: string;
  by: string;
  at: number;
}

/** Note on the event that you invited `agent`, so every organizer sees it in Invited. The inbox
 * invite itself is sent separately and stays encrypted to them. */
export async function recordInvite(id: string, agent: string): Promise<void> {
  const { cell } = await cellOf(id);
  await callCell<null>(cell, 'project', 'record_invite', stringToHash(agent));
}

/** Everyone invited to this event, oldest first (members included: filter them out to show
 * who hasn't joined yet). */
export async function eventInvites(id: string): Promise<EventInvite[]> {
  const { cell } = await cellOf(id);
  const wire = await callCell<{ agent: AgentPubKey; by: AgentPubKey; at: number }[]>(
    cell,
    'project',
    'event_invites',
    null,
  );
  return wire.map((i) => ({
    agent: hashToString(i.agent),
    by: hashToString(i.by),
    at: i.at / 1000,
  }));
}
