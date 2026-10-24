import {
  CALL_PAYLOAD_MAX,
  CALL_SIGNAL_TYPES,
  type CallChannel,
  type CallEnvelope,
} from '#lib/domain/calls.ts';
import { acceptSignals, onSignal, sendSignal } from '#lib/holo/index.ts';

import { chatSummary, myChats } from './messages.ts';
import { myAgent, namesOf } from './people.ts';

/**
 * web's call socket (`/calls/ws`) as direct signals between the two people's nodes
 * (ADR 0050). web's server checked that both are in the direct chat; here the receiving side
 * does: a signal counts only from the other person in one of your direct chats.
 */

/** Direct chat seed → the other person, once confirmed. */
const peers = new Map<string, Promise<string | null>>();
const names = new Map<string, Promise<string>>();

async function findPeer(seed: string): Promise<string | null> {
  if (!(await myChats()).some((c) => c.seed === seed)) return null;
  const [chat, me] = await Promise.all([chatSummary(seed, 0), myAgent()]);
  if (chat.kind !== 'direct' || !chat.member) return null;
  const others = chat.members.filter((m) => m !== me);
  return others.length === 1 ? (others[0] ?? null) : null;
}

/** The other person in a direct chat, or null (not yours, a group, or not found yet). */
export function directPeer(seed: string): Promise<string | null> {
  const cached = peers.get(seed);
  if (cached) return cached;
  const finding = findPeer(seed).catch(() => null);
  peers.set(seed, finding);
  // Only a confirmed peer is kept: an invitation may still be on its way.
  void finding.then((peer) => peer ?? peers.delete(seed));
  return finding;
}

function nameOf(agent: string): Promise<string> {
  let found = names.get(agent);
  if (!found) {
    found = namesOf([agent]).then(
      (all) => all[agent] ?? 'Someone',
      () => 'Someone',
    );
    names.set(agent, found);
  }
  return found;
}

function parse(data: Uint8Array): CallEnvelope | null {
  if (data.length > CALL_PAYLOAD_MAX * 2) return null;
  let raw: unknown;
  try {
    raw = JSON.parse(new TextDecoder().decode(data));
  } catch {
    return null;
  }
  if (!raw || typeof raw !== 'object') return null;
  const { callId, conversationId, type, payload } = raw as Record<string, unknown>;
  const kind = CALL_SIGNAL_TYPES.find((t) => t === type);
  if (typeof callId !== 'string' || callId.length > 64 || !kind) return null;
  if (typeof conversationId !== 'string') return null;
  const body = payload && typeof payload === 'object' ? (payload as Record<string, unknown>) : {};
  if (JSON.stringify(body).length > CALL_PAYLOAD_MAX) return null;
  return { callId, conversationId, type: kind, payload: body };
}

async function receive(
  from: string,
  data: Uint8Array,
  onEnvelope: (envelope: CallEnvelope) => void,
): Promise<void> {
  const envelope = parse(data);
  if (!envelope || (await directPeer(envelope.conversationId)) !== from) return;
  onEnvelope({ ...envelope, fromUserId: from, fromUsername: await nameOf(from) });
}

async function deliver(envelope: CallEnvelope): Promise<void> {
  const peer = await directPeer(envelope.conversationId);
  if (!peer) return;
  const { callId, conversationId, type, payload } = envelope;
  const wire = JSON.stringify({ callId, conversationId, type, payload: payload ?? {} });
  await sendSignal(peer, new TextEncoder().encode(wire));
}

/** Start taking calls: grant other nodes the right to signal this inbox, then listen. */
export function openCallChannel(onEnvelope: (envelope: CallEnvelope) => void): CallChannel {
  void acceptSignals().catch(() => undefined);
  const stop = onSignal((from, data) => void receive(from, data, onEnvelope).catch(() => {}));
  return {
    send(envelope) {
      void deliver(envelope).catch(() => undefined);
    },
    close: stop,
  };
}
