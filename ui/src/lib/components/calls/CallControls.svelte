<script lang="ts">
  import type { CallView } from '#lib/domain/calls.ts';

  /** The call screen's buttons, ported from web/src/lib/calls/CallHost.svelte: Decline and
   * Accept while ringing; Mute, Speaker (where the browser can pick an output) and End during
   * a call; Close once it is over. */
  interface Props {
    view: CallView;
    live: boolean;
    speaker: boolean;
    onAccept: () => void;
    onDecline: () => void;
    onMute: () => void;
    onSpeaker: () => void;
    onHangUp: () => void;
    onDismiss: () => void;
  }

  let { view, live, speaker, onAccept, onDecline, onMute, onSpeaker, onHangUp, onDismiss }: Props =
    $props();
</script>

<div class="call-controls">
  {#if view.phase === 'ringing'}
    <div class="control">
      <button class="round decline" type="button" aria-label="Decline" onclick={onDecline}>
        <svg aria-hidden="true" viewBox="0 0 24 24" width="30" height="30">
          <path
            fill="currentColor"
            d="M12 9c-1.6 0-3.15.25-4.6.72v3.1c0 .39-.23.74-.56.9-.98.49-1.87 1.12-2.66 1.85-.18.18-.43.28-.7.28-.28 0-.53-.11-.71-.29L.29 13.08a.956.956 0 0 1 0-1.36C3.34 8.78 7.46 7 12 7s8.66 1.78 11.71 4.72c.18.18.29.44.29.71 0 .28-.11.53-.29.71l-2.48 2.48c-.18.18-.43.29-.71.29-.27 0-.52-.11-.7-.28-.79-.74-1.69-1.36-2.67-1.85-.33-.16-.56-.5-.56-.9v-3.1C15.15 9.25 13.6 9 12 9z"
          />
        </svg>
      </button>
      <span>Decline</span>
    </div>
    <div class="control">
      <button
        class="round accept"
        type="button"
        aria-label="Accept"
        disabled={view.connecting}
        onclick={onAccept}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24" width="30" height="30">
          <path
            fill="currentColor"
            d="M6.6 10.8c1.4 2.8 3.8 5.1 6.6 6.6l2.2-2.2c.3-.3.7-.4 1-.2 1.1.4 2.3.6 3.6.6.6 0 1 .4 1 1V20c0 .6-.4 1-1 1-9.4 0-17-7.6-17-17 0-.6.4-1 1-1h3.5c.6 0 1 .4 1 1 0 1.3.2 2.5.6 3.6.1.3 0 .7-.2 1L6.6 10.8z"
          />
        </svg>
      </button>
      <span>Accept</span>
    </div>
  {:else if live}
    <div class="control">
      <button
        class="round"
        type="button"
        aria-pressed={view.muted}
        aria-label={view.muted ? 'Unmute' : 'Mute'}
        onclick={onMute}
      >
        {#if view.muted}
          <svg aria-hidden="true" viewBox="0 0 24 24" width="28" height="28">
            <path
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              d="M3 3l18 18M9 9v3a3 3 0 0 0 5.1 2.1M15 9.3V5a3 3 0 0 0-5.9-.7M19 11a7 7 0 0 1-1.1 3.8M5 11a7 7 0 0 0 10.6 6M12 18v3"
            />
          </svg>
        {:else}
          <svg aria-hidden="true" viewBox="0 0 24 24" width="28" height="28">
            <path
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              d="M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3zM19 11a7 7 0 0 1-14 0M12 18v3"
            />
          </svg>
        {/if}
      </button>
      <span>{view.muted ? 'Unmute' : 'Mute'}</span>
    </div>
    {#if speaker}
      <div class="control">
        <button
          class="round"
          type="button"
          aria-pressed={view.speaker}
          aria-label={view.speaker ? 'Speaker off' : 'Speaker on'}
          onclick={onSpeaker}
        >
          <svg aria-hidden="true" viewBox="0 0 24 24" width="28" height="28">
            <path fill="currentColor" d="M4 9v6h4l5 4V5L8 9H4z" />
            <path
              d="M16 8.5a5 5 0 0 1 0 7M18.5 6a8.5 8.5 0 0 1 0 12"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
            />
          </svg>
        </button>
        <span>Speaker</span>
      </div>
    {/if}
    <div class="control">
      <button class="round decline" type="button" aria-label="End call" onclick={onHangUp}>
        <svg aria-hidden="true" viewBox="0 0 24 24" width="30" height="30">
          <path
            fill="currentColor"
            d="M12 9c-1.6 0-3.15.25-4.6.72v3.1c0 .39-.23.74-.56.9-.98.49-1.87 1.12-2.66 1.85-.18.18-.43.28-.7.28-.28 0-.53-.11-.71-.29L.29 13.08a.956.956 0 0 1 0-1.36C3.34 8.78 7.46 7 12 7s8.66 1.78 11.71 4.72c.18.18.29.44.29.71 0 .28-.11.53-.29.71l-2.48 2.48c-.18.18-.43.29-.71.29-.27 0-.52-.11-.7-.28-.79-.74-1.69-1.36-2.67-1.85-.33-.16-.56-.5-.56-.9v-3.1C15.15 9.25 13.6 9 12 9z"
          />
        </svg>
      </button>
      <span>End</span>
    </div>
  {:else}
    <div class="control">
      <button class="round" type="button" aria-label="Close" onclick={onDismiss}>
        <svg aria-hidden="true" viewBox="0 0 24 24" width="26" height="26">
          <path
            d="M6 6l12 12M18 6L6 18"
            fill="none"
            stroke="currentColor"
            stroke-width="2.4"
            stroke-linecap="round"
          />
        </svg>
      </button>
      <span>Close</span>
    </div>
  {/if}
</div>
