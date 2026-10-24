import type { CallEnvelope } from '#lib/domain/calls.ts';

import { isLive, type CallSession } from './callSession.svelte.ts';

/** web's session.ts `handleEnvelope`: what each signal from the other person does. */

function descriptionFrom(
  payload: Record<string, unknown> | null | undefined,
): RTCSessionDescriptionInit | null {
  const sdp = payload?.['sdp'];
  const type = payload?.['type'];
  if (typeof sdp !== 'string' || (type !== 'offer' && type !== 'answer')) return null;
  return { type, sdp };
}

function candidateFrom(
  payload: Record<string, unknown> | null | undefined,
): RTCIceCandidateInit | null {
  const candidate = payload?.['candidate'];
  if (!candidate || typeof candidate !== 'object') return null;
  return candidate as RTCIceCandidateInit;
}

/** Any failure while setting up audio ends the call on both phones. */
async function orHangUp(session: CallSession, run: () => Promise<void>): Promise<void> {
  try {
    await run();
  } catch {
    session.send('hangup');
    session.reset('ended');
  }
}

function invited(session: CallSession, envelope: CallEnvelope): void {
  // The caller repeats its invite until this device answers; answer again, ring once.
  if (session.hasFinished(envelope.callId)) return;
  if (envelope.callId === session.view.callId) {
    if (session.view.phase === 'ringing') session.send('ringing');
    return;
  }
  if (isLive(session.view.phase)) {
    session.reply(envelope, 'reject');
    return;
  }
  session.role = 'callee';
  session.open(
    'ringing',
    envelope.callId,
    envelope.conversationId,
    envelope.fromUsername || 'Someone',
  );
  session.send('ringing');
}

function accepted(session: CallSession): void {
  session.clearTimers();
  session.patch({ connecting: true });
  void orHangUp(session, async () => {
    const offer = await (await session.ensureMedia()).createOffer();
    session.send('offer', { type: offer.type, sdp: offer.sdp });
  });
}

function offered(session: CallSession, envelope: CallEnvelope): void {
  void orHangUp(session, async () => {
    const media = session.media ?? (await session.ensureMedia());
    const offer = descriptionFrom(envelope.payload);
    if (!offer) return;
    const answer = await media.acceptOffer(offer);
    session.send('answer', { type: answer.type, sdp: answer.sdp });
    session.patch({ phase: 'active', connectedAt: Date.now(), connecting: false });
  });
}

function answered(session: CallSession, envelope: CallEnvelope): void {
  const answer = descriptionFrom(envelope.payload);
  const media = session.media;
  if (!answer || !media) return;
  void orHangUp(session, async () => {
    await media.acceptAnswer(answer);
    session.patch({ phase: 'active', connectedAt: Date.now(), connecting: false });
  });
}

export function handleEnvelope(session: CallSession, envelope: CallEnvelope): void {
  if (envelope.type === 'invite') {
    invited(session, envelope);
    return;
  }
  if (!envelope.callId || envelope.callId !== session.view.callId) return;
  const caller = session.role === 'caller';
  switch (envelope.type) {
    case 'ringing':
      if (caller) session.ringConfirmed = true;
      return;
    case 'reject':
      session.reset('declined');
      return;
    case 'hangup':
      session.reset('ended');
      return;
    case 'accept':
      if (caller) accepted(session);
      return;
    case 'offer':
      // Only the device that pressed Accept answers; another of the callee's keeps ringing.
      if (session.role === 'callee' && session.view.connecting) offered(session, envelope);
      return;
    case 'answer':
      if (caller) answered(session, envelope);
      return;
    case 'ice': {
      const candidate = candidateFrom(envelope.payload);
      if (candidate && session.media) {
        void session.media.addIceCandidate(candidate).catch(() => {
          /* A late candidate after hangup is ignored. */
        });
      }
    }
  }
}
