import type { CallChannel, CallEnvelope, CallPhase, CallView } from '#lib/domain/calls.ts';
import type { CallLine } from '#lib/domain/chat.ts';
import { configuredIceServers, createCallMedia, type CallMedia } from '#lib/services/callMedia.ts';
import { openCallChannel, rememberPeer } from '#lib/services/calls.ts';
import { logChatCall } from '#lib/services/messages.ts';

import { handleEnvelope } from './callSignals.ts';

/** web/src/lib/calls/session.ts: one direct call at a time, signalled node to node (ADR 0050). */

/** Ring this long before the caller gives up, as phone apps do (web's NO_ANSWER_MS). */
const NO_ANSWER_MS = 45_000;
/** App-only: with no `ringing` back by then, the other person doesn't have the app open. */
const RING_ACK_MS = 15_000;
/** App-only: direct signals are best effort (one sent while the other page reloads is lost),
 * so the invite repeats this often until their device answers `ringing`. */
const INVITE_REPEAT_MS = 2_500;

const idleView: CallView = {
  phase: 'idle',
  callId: null,
  conversationId: null,
  peerName: '',
  muted: false,
  connectedAt: null,
  notice: null,
  connecting: false,
  speaker: false,
  lastDurationMs: null,
};

export const isLive = (phase: CallPhase): boolean =>
  phase === 'calling' || phase === 'ringing' || phase === 'active';

export class CallSession {
  view = $state<CallView>({ ...idleView });
  role: 'caller' | 'callee' | null = null;
  media: CallMedia | null = null;
  private channel: CallChannel | null = null;
  private attempt = 0;
  private timers: ReturnType<typeof setTimeout>[] = [];
  /** Set when the callee's device answered the invite with `ringing`. */
  ringConfirmed = false;
  /** Calls that ended here, so a repeated invite still on its way doesn't ring again. */
  private finished: string[] = [];

  hasFinished(callId: string): boolean {
    return this.finished.includes(callId);
  }

  patch(next: Partial<CallView>): void {
    this.view = { ...this.view, ...next };
  }

  /** The screen for a new call (ringing, or calling someone). */
  open(phase: CallPhase, callId: string, conversationId: string, peerName: string): void {
    this.view = { ...idleView, phase, callId, conversationId, peerName };
  }

  clearTimers(): void {
    for (const timer of this.timers) clearTimeout(timer);
    this.timers = [];
  }

  /** web's `logFinishedCall`: one line per call in the chat. A call that connected is
   * reported by both phones (the first report shows); one that didn't, by the caller. */
  private logLine(phase: CallPhase): void {
    const { callId, conversationId, connectedAt } = this.view;
    if (!callId || !conversationId) return;
    let outcome: CallLine['outcome'] | null = null;
    if (connectedAt) outcome = 'completed';
    else if (this.role === 'caller') outcome = phase === 'declined' ? 'declined' : 'missed';
    if (!outcome) return;
    const seconds = connectedAt ? (Date.now() - connectedAt) / 1000 : 0;
    void logChatCall(conversationId, callId, outcome, seconds).catch(() => {
      /* The other phone may still log it. */
    });
  }

  reset(phase: CallPhase = 'idle', notice: string | null = null, logLine = true): void {
    if (logLine && isLive(this.view.phase)) this.logLine(phase);
    if (this.view.callId) this.finished = [...this.finished.slice(-19), this.view.callId];
    this.attempt += 1;
    this.clearTimers();
    this.media?.stop();
    this.media = null;
    this.role = null;
    const lastDurationMs = this.view.connectedAt ? Date.now() - this.view.connectedAt : null;
    this.view = { ...idleView, phase, peerName: this.view.peerName, notice, lastDurationMs };
  }

  send(type: CallEnvelope['type'], payload?: Record<string, unknown> | null): void {
    const { callId, conversationId } = this.view;
    if (!this.channel || !callId || !conversationId) return;
    this.channel.send({ callId, conversationId, type, payload: payload ?? {} });
  }

  /** Answer an envelope that isn't for the current call (web's auto-reject while busy). */
  reply(to: CallEnvelope, type: CallEnvelope['type']): void {
    this.channel?.send({ callId: to.callId, conversationId: to.conversationId, type, payload: {} });
  }

  async ensureMedia(): Promise<CallMedia> {
    if (this.media) return this.media;
    const next = createCallMedia(configuredIceServers(), (candidate) => {
      this.send('ice', { candidate });
    });
    this.media = next;
    await next.startLocalAudio();
    return next;
  }

  connect(): void {
    if (this.channel) return;
    this.channel = openCallChannel((envelope) => handleEnvelope(this, envelope));
  }

  disconnect(): void {
    this.attempt += 1;
    this.clearTimers();
    if (isLive(this.view.phase)) this.send('hangup');
    this.media?.stop();
    this.media = null;
    this.channel?.close();
    this.channel = null;
    this.role = null;
    this.view = { ...idleView };
  }

  async placeDirectCall(
    conversationId: string,
    peerName: string,
    peer: string | null = null,
  ): Promise<void> {
    if (peer) rememberPeer(conversationId, peer);
    this.connect();
    if (isLive(this.view.phase)) return;
    const callId = crypto.randomUUID();
    const attempt = this.attempt;
    this.role = 'caller';
    this.open('calling', callId, conversationId, peerName);
    try {
      await this.ensureMedia();
    } catch {
      // Nothing rang, so this isn't a missed call; leave no call line in the chat.
      this.reset('ended', 'Microphone unavailable. Allow microphone access to call.', false);
      return;
    }
    if (attempt !== this.attempt) return;
    this.ringConfirmed = false;
    this.send('invite', {});
    this.clearTimers();
    for (let at = INVITE_REPEAT_MS; at < RING_ACK_MS; at += INVITE_REPEAT_MS) {
      this.timers.push(
        setTimeout(() => {
          const unanswered = this.view.callId === callId && this.view.phase === 'calling';
          if (unanswered && !this.ringConfirmed) this.send('invite', {});
        }, at),
      );
    }
    const giveUp = (phase: CallPhase, notice: string | null) => () => {
      if (this.view.callId !== callId || this.view.phase !== 'calling') return;
      if (phase === 'unavailable' && this.ringConfirmed) return;
      this.send('hangup');
      this.reset(phase, notice);
    };
    this.timers.push(
      setTimeout(giveUp('unavailable', null), RING_ACK_MS),
      setTimeout(giveUp('ended', 'No answer'), NO_ANSWER_MS),
    );
  }

  async acceptCall(): Promise<void> {
    if (this.view.phase !== 'ringing' || this.role !== 'callee') return;
    try {
      await this.ensureMedia();
    } catch {
      this.send('reject');
      this.reset('ended', 'Microphone unavailable');
      return;
    }
    this.patch({ connecting: true });
    this.send('accept', {});
  }

  declineCall(): void {
    if (this.view.phase !== 'ringing') return;
    this.send('reject');
    this.reset('idle');
  }

  hangUpCall(): void {
    if (!isLive(this.view.phase)) return;
    const wasActive = this.view.phase === 'active';
    this.send('hangup');
    this.reset(wasActive ? 'ended' : 'idle');
  }

  toggleMute(): void {
    if (!this.media || (this.view.phase !== 'active' && this.view.phase !== 'calling')) return;
    const muted = !this.view.muted;
    this.media.setMuted(muted);
    this.patch({ muted });
  }

  async toggleSpeaker(): Promise<void> {
    if (!this.media || (this.view.phase !== 'active' && this.view.phase !== 'calling')) return;
    const next = !this.view.speaker;
    if (await this.media.setSpeaker(next)) this.patch({ speaker: next });
  }

  dismiss(): void {
    if (this.view.phase !== 'idle' && !isLive(this.view.phase)) this.view = { ...idleView };
  }
}

/** The page's one call session; the call screen drives it, Messages starts calls with it. */
export const calls = new CallSession();
