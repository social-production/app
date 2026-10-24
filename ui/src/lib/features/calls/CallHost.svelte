<script lang="ts">
  import CallControls from '#lib/components/calls/CallControls.svelte';
  import { callStatusLine, formatElapsed } from '#lib/domain/calls.ts';
  import { canSelectAudioOutput } from '#lib/services/callMedia.ts';
  import { startCallTone, type CallTone } from '#lib/services/callTones.ts';
  import { callRequest } from '#lib/stores/callRequest.svelte.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { calls, isLive } from './callSession.svelte.ts';

  import '#lib/components/styles/calls.css';

  /** web/src/lib/calls/CallHost.svelte: the full-screen call (ring, ringback, timer) and its
   * "Tap to return" pill. Mounted once by the root layout; takes calls while signed in. */

  const speakerSupported = canSelectAudioOutput();
  const ENDED_DISMISS_MS = 2500;

  let now = $state(Date.now());
  let minimized = $state(false);

  const view = $derived(calls.view);
  const visible = $derived(view.phase !== 'idle');
  const live = $derived(isLive(view.phase));
  const finished = $derived(visible && !live);
  const elapsed = $derived(formatElapsed(view.connectedAt ? now - view.connectedAt : 0));
  const status = $derived(callStatusLine(view, elapsed));
  const initials = $derived(
    (view.peerName || '?')
      .replace(/[^a-z0-9]/gi, '')
      .slice(0, 2)
      .toUpperCase() || '?',
  );

  const signedIn = $derived(connection.current.status === 'connected');
  $effect(() => {
    if (!signedIn) return;
    calls.connect();
    return () => calls.disconnect();
  });

  $effect(() => {
    const timer = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(timer);
  });

  // A call asked for from a direct chat's header.
  $effect(() => {
    const next = callRequest.next;
    if (!next) return;
    callRequest.next = null;
    void calls.placeDirectCall(next.seed, next.name);
  });

  $effect(() => {
    if (!live) minimized = false;
  });

  // Ring for the person being called; ringback for the caller until they answer.
  const wanted = $derived(
    view.connecting
      ? null
      : view.phase === 'ringing'
        ? 'ring'
        : view.phase === 'calling'
          ? 'ringback'
          : null,
  );
  $effect(() => {
    if (!wanted) return;
    const tone: CallTone = startCallTone(wanted);
    return () => tone.stop();
  });

  // A problem notice (e.g. the microphone) stays long enough to read; it can be closed sooner.
  const dismissAfter = $derived(
    finished ? (view.notice ? ENDED_DISMISS_MS * 3 : ENDED_DISMISS_MS) : null,
  );
  $effect(() => {
    if (dismissAfter === null) return;
    const timer = setTimeout(() => calls.dismiss(), dismissAfter);
    return () => clearTimeout(timer);
  });
</script>

<div class="call-host">
  {#if visible && minimized}
    <button class="call-pill" type="button" onclick={() => (minimized = false)}>
      <span class="pill-dot" aria-hidden="true"></span>
      <span class="pill-text">
        <strong>{view.peerName || 'Call'}</strong>
        <span>{view.phase === 'active' ? elapsed : status}</span>
      </span>
      <span class="pill-return">Tap to return</span>
    </button>
  {:else if visible}
    <div class="call-screen" role="dialog" aria-modal="true" aria-labelledby="call-title">
      <div class="call-top">
        {#if live && view.phase !== 'ringing'}
          <button
            class="icon-plain"
            type="button"
            aria-label="Back to the app"
            title="Back to the app"
            onclick={() => (minimized = true)}
          >
            <svg aria-hidden="true" viewBox="0 0 24 24" width="26" height="26">
              <path
                d="M6 9l6 6 6-6"
                fill="none"
                stroke="currentColor"
                stroke-width="2.4"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </button>
        {:else}
          <span></span>
        {/if}
        <span class="call-kind">Voice call</span>
        <span></span>
      </div>

      <div class="call-identity">
        <div
          class="avatar"
          class:pulsing={view.phase === 'calling' || view.phase === 'ringing'}
          aria-hidden="true"
        >
          {initials}
        </div>
        <h2 id="call-title">{view.peerName || 'Someone'}</h2>
        <p class="call-status" aria-live="polite">{status}</p>
      </div>

      <CallControls
        {view}
        {live}
        speaker={speakerSupported}
        onAccept={() => void calls.acceptCall()}
        onDecline={() => calls.declineCall()}
        onMute={() => calls.toggleMute()}
        onSpeaker={() => void calls.toggleSpeaker()}
        onHangUp={() => calls.hangUpCall()}
        onDismiss={() => calls.dismiss()}
      />
    </div>
  {/if}
</div>
