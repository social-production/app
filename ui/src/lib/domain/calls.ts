/**
 * web's call signalling envelope (web/src/lib/calls/types.ts). Audio never travels in it;
 * here it goes from node to node as a direct signal (ADR 0050).
 */
export type CallSignalType =
  | 'invite'
  /** App-only: the callee's device confirms the ring. web's server knew who was online;
   * a node can't tell whether its person has the app open, so no ack means unavailable. */
  | 'ringing'
  | 'accept'
  | 'reject'
  | 'offer'
  | 'answer'
  | 'ice'
  | 'hangup';

export interface CallEnvelope {
  callId: string;
  /** The direct chat's seed. */
  conversationId: string;
  type: CallSignalType;
  payload?: Record<string, unknown> | null;
  /** Set on arrival from the sender's key; never trusted from the payload. */
  fromUserId?: string;
  fromUsername?: string;
}

export interface CallChannel {
  send(envelope: CallEnvelope): void;
  close(): void;
}

export const CALL_SIGNAL_TYPES: readonly CallSignalType[] = [
  'invite',
  'ringing',
  'accept',
  'reject',
  'offer',
  'answer',
  'ice',
  'hangup',
];

/** web-backend's MAX_PAYLOAD_CHARS: an SDP offer fits easily. */
export const CALL_PAYLOAD_MAX = 65_536;

export type CallPhase =
  'idle' | 'calling' | 'ringing' | 'active' | 'unavailable' | 'busy' | 'declined' | 'ended';

export interface CallView {
  phase: CallPhase;
  callId: string | null;
  conversationId: string | null;
  peerName: string;
  muted: boolean;
  connectedAt: number | null;
  notice: string | null;
  /** True between Accept and audio flowing. */
  connecting: boolean;
  speaker: boolean;
  /** Length of the call that just ended, for the "Call ended" line. */
  lastDurationMs: number | null;
}

export function formatElapsed(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}

/** web's CallHost `statusLine`: the line under the other person's name. */
export function callStatusLine(view: CallView, elapsed: string): string {
  if (view.connecting) return 'Connecting…';
  switch (view.phase) {
    case 'calling':
      return 'Ringing…';
    case 'ringing':
      return 'Incoming voice call';
    case 'active':
      return elapsed;
    case 'unavailable':
      return 'Not available right now';
    case 'busy':
      return 'On another call';
    case 'declined':
      return 'Call declined';
    default:
      if (view.notice) return view.notice;
      return view.lastDurationMs
        ? `Call ended · ${formatElapsed(view.lastDurationMs)}`
        : 'Call ended';
  }
}
