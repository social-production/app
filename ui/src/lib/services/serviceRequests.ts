import type { RequestSettings } from '#lib/domain/projects.ts';
import type { RequestStatus, ServiceRequest } from '#lib/domain/serviceRequests.ts';
import {
  callCell,
  conversationCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { propose } from './governance.ts';
import { chatSummary, myChats, startChat } from './messages.ts';
import { myAgent } from './people.ts';

/**
 * Service requests (ADR 0076, web's personal-service requests). A request lives in the direct
 * chat between the person asking and the service's creator, so only those two see it, and it
 * posts web's request and status lines into that chat. The creator's requests are read from
 * each of their direct chats.
 */
interface RequestWire {
  id: ActionHash;
  project: string;
  service: string;
  requester: AgentPubKey;
  title: string;
  body: string;
  status: RequestStatus;
  decline_reason: string | null;
  starts_at: number | null;
  ends_at: number | null;
  created_at: number;
  updated_at: number;
}

const chat = async <T>(seed: string, fn: string, payload?: unknown): Promise<T> =>
  callCell<T>(await conversationCell(seed), 'messages', fn, payload);

/** Your direct chats, with the other person in each. */
async function directChats(): Promise<{ seed: string; other: string }[]> {
  const me = await myAgent();
  const all = await Promise.all(
    (await myChats()).map((c) => chatSummary(c.seed, c.at).catch(() => null)),
  );
  return all.flatMap((c) => {
    if (!c || c.kind !== 'direct' || !c.member || c.members.length !== 2) return [];
    const other = c.members.find((m) => m !== me);
    return other ? [{ seed: c.id, other }] : [];
  });
}

function toRequest(w: RequestWire, seed: string, other: string): ServiceRequest {
  return {
    id: hashToString(w.id),
    chat: seed,
    other,
    project: w.project,
    service: w.service,
    requester: hashToString(w.requester),
    title: w.title,
    body: w.body,
    status: w.status,
    declineReason: w.decline_reason,
    startsAt: w.starts_at,
    endsAt: w.ends_at,
    createdAt: w.created_at,
    updatedAt: w.updated_at,
  };
}

/** Every request in your direct chats (sent or received), optionally for one service. */
export async function myServiceRequests(project?: string): Promise<ServiceRequest[]> {
  const chats = await directChats();
  const lists = await Promise.all(
    chats.map(async ({ seed, other }) => {
      const wire = await chat<RequestWire[]>(seed, 'service_requests').catch(() => []);
      return wire.map((w) => toRequest(w, seed, other));
    }),
  );
  return lists
    .flat()
    .filter((r) => !project || r.project === project)
    .sort((a, b) => b.createdAt - a.createdAt);
}

/** The direct chat with `agent`, started (with no first message) when there is none. */
async function directChatWith(agent: string): Promise<string> {
  const existing = (await directChats()).find((c) => c.other === agent);
  return existing?.seed ?? startChat('direct', [agent], null, null);
}

export interface RequestDraft {
  title: string;
  body: string;
  startsAt: number | null;
  endsAt: number | null;
}

/** web's `create_service_request` on a personal service: opens the chat with its creator. */
export async function sendServiceRequest(
  project: { id: string; title: string; creator: string },
  draft: RequestDraft,
): Promise<{ chat: string; id: string }> {
  const seed = await directChatWith(project.creator);
  const id = await chat<ActionHash>(seed, 'send_service_request', {
    project: project.id,
    service: project.title,
    title: draft.title.trim(),
    body: draft.body.trim(),
    starts_at: draft.startsAt,
    ends_at: draft.endsAt,
  });
  return { chat: seed, id: hashToString(id) };
}

/** web's `update_service_request_status`: accept, decline (with a reason) or cancel. */
export async function decideServiceRequest(
  request: Pick<ServiceRequest, 'chat' | 'id'>,
  status: Exclude<RequestStatus, 'open'>,
  reason = '',
): Promise<void> {
  await chat<RequestWire>(request.chat, 'decide_service_request', {
    request: stringToHash(request.id),
    status,
    reason: status === 'declined' ? reason.trim() : '',
  });
}

/** web's personal-service settings save (applied at once; a collective service votes). */
export async function saveRequestSettings(
  project: string,
  settings: RequestSettings,
  reason = '',
): Promise<void> {
  await propose(project, {
    RequestSettings: {
      enabled: settings.enabled,
      mode: settings.mode,
      off_schedule: settings.offSchedule,
      reason: reason.trim(),
    },
  });
}
