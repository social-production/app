/**
 * Device audio for a one-to-one call, ported from web/src/lib/calls/media.ts (ADR 0050).
 * The call screen never calls getUserMedia or RTCPeerConnection itself; the native app's
 * WebView uses these same APIs.
 */

export type IceCandidateHandler = (candidate: RTCIceCandidateInit) => void;

export interface CallMedia {
  startLocalAudio(): Promise<void>;
  createOffer(): Promise<RTCSessionDescriptionInit>;
  acceptOffer(offer: RTCSessionDescriptionInit): Promise<RTCSessionDescriptionInit>;
  acceptAnswer(answer: RTCSessionDescriptionInit): Promise<void>;
  addIceCandidate(candidate: RTCIceCandidateInit): Promise<void>;
  setMuted(muted: boolean): void;
  /** Route the other person's voice to the loudspeaker (true) or the default output. */
  setSpeaker(on: boolean): Promise<boolean>;
  stop(): void;
}

/**
 * Browsers only let a page pick the audio output where `setSinkId` exists (Chrome, Edge,
 * Android Chrome). Safari on iPhone does not; that needs the native wrap.
 */
export function canSelectAudioOutput(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof HTMLMediaElement !== 'undefined' &&
    'setSinkId' in HTMLMediaElement.prototype &&
    typeof navigator.mediaDevices?.enumerateDevices === 'function'
  );
}

const list = (value: unknown): string[] =>
  String(value ?? '')
    .split(',')
    .map((url) => url.trim())
    .filter(Boolean);

/** web's VITE_CALL_* settings: STUN (Google's by default) and optional TURN relays. */
export function configuredIceServers(): RTCIceServer[] {
  const env = import.meta.env;
  const stun = list(env.VITE_CALL_STUN_URLS);
  const servers: RTCIceServer[] = (stun.length ? stun : ['stun:stun.l.google.com:19302']).map(
    (urls) => ({ urls }),
  );
  const username = String(env.VITE_CALL_TURN_USERNAME ?? '').trim();
  const credential = String(env.VITE_CALL_TURN_CREDENTIAL ?? '').trim();
  for (const urls of list(env.VITE_CALL_TURN_URLS)) {
    servers.push({
      urls,
      ...(username ? { username } : {}),
      ...(credential ? { credential } : {}),
    });
  }
  return servers;
}

class PeerAudio implements CallMedia {
  private readonly peer: RTCPeerConnection;
  private readonly remoteAudio = new Audio();
  private localStream: MediaStream | null = null;
  private remoteDescriptionSet = false;
  private readonly pendingCandidates: RTCIceCandidateInit[] = [];
  private stopped = false;

  constructor(iceServers: RTCIceServer[], onIceCandidate: IceCandidateHandler) {
    this.peer = new RTCPeerConnection({ iceServers });
    this.remoteAudio.autoplay = true;
    this.remoteAudio.setAttribute('playsinline', '');
    this.peer.onicecandidate = (event) => {
      if (this.stopped || !event.candidate) return;
      onIceCandidate(event.candidate.toJSON());
    };
    this.peer.ontrack = (event) => {
      const [stream] = event.streams;
      if (!stream) return;
      this.remoteAudio.srcObject = stream;
      void this.remoteAudio.play().catch(() => {
        /* Playback is primed during the accept or call gesture. */
      });
    };
  }

  private async flushCandidates(): Promise<void> {
    this.remoteDescriptionSet = true;
    for (const candidate of this.pendingCandidates.splice(0)) {
      await this.peer.addIceCandidate(candidate);
    }
  }

  async startLocalAudio(): Promise<void> {
    this.localStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
    for (const track of this.localStream.getTracks()) {
      this.peer.addTrack(track, this.localStream);
    }
    void this.remoteAudio.play().catch(() => {
      /* No remote stream yet. ontrack retries playback. */
    });
  }

  async createOffer(): Promise<RTCSessionDescriptionInit> {
    const offer = await this.peer.createOffer();
    await this.peer.setLocalDescription(offer);
    return this.peer.localDescription?.toJSON() ?? offer;
  }

  async acceptOffer(offer: RTCSessionDescriptionInit): Promise<RTCSessionDescriptionInit> {
    await this.peer.setRemoteDescription(offer);
    await this.flushCandidates();
    const answer = await this.peer.createAnswer();
    await this.peer.setLocalDescription(answer);
    return this.peer.localDescription?.toJSON() ?? answer;
  }

  async acceptAnswer(answer: RTCSessionDescriptionInit): Promise<void> {
    await this.peer.setRemoteDescription(answer);
    await this.flushCandidates();
  }

  async addIceCandidate(candidate: RTCIceCandidateInit): Promise<void> {
    if (!this.remoteDescriptionSet) {
      this.pendingCandidates.push(candidate);
      return;
    }
    await this.peer.addIceCandidate(candidate);
  }

  setMuted(muted: boolean): void {
    for (const track of this.localStream?.getAudioTracks() ?? []) {
      track.enabled = !muted;
    }
  }

  async setSpeaker(on: boolean): Promise<boolean> {
    if (!canSelectAudioOutput()) return false;
    const sinkable = this.remoteAudio as HTMLAudioElement & {
      setSinkId(id: string): Promise<void>;
    };
    try {
      const outputs = (await navigator.mediaDevices.enumerateDevices()).filter(
        (device) => device.kind === 'audiooutput',
      );
      const speaker = outputs.find((device) => /speaker/i.test(device.label));
      const earpiece = outputs.find((device) => /earpiece|receiver|handset/i.test(device.label));
      const target = on ? (speaker?.deviceId ?? 'default') : (earpiece?.deviceId ?? 'default');
      await sinkable.setSinkId(target);
      return true;
    } catch {
      return false;
    }
  }

  stop(): void {
    this.stopped = true;
    this.peer.onicecandidate = null;
    this.peer.ontrack = null;
    for (const track of this.localStream?.getTracks() ?? []) {
      track.stop();
    }
    this.localStream = null;
    this.remoteAudio.srcObject = null;
    this.peer.close();
  }
}

export function createCallMedia(
  iceServers: RTCIceServer[],
  onIceCandidate: IceCandidateHandler,
): CallMedia {
  return new PeerAudio(iceServers, onIceCandidate);
}
