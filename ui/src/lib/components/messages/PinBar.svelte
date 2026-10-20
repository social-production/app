<script lang="ts">
  import type { ChatPin } from '#lib/domain/messages.ts';

  /**
   * web's Telegram-style pin bar: shows the newest pin; each tap jumps to it and moves on to
   * the next older one. People who may pin can unpin from here.
   */
  interface Props {
    pins: ChatPin[];
    canUnpin: boolean;
    onJump: (messageId: string) => void;
    onUnpin: (messageId: string) => void;
  }

  let { pins, canUnpin, onJump, onUnpin }: Props = $props();

  let cursor = $state(0);
  const index = $derived(cursor % pins.length);
  const current = $derived(pins[pins.length - 1 - index]);
</script>

{#if current}
  <div class="pin-bar" aria-label="Pinned messages">
    <span class="pin-rail" aria-hidden="true">
      {#each pins as pin, i (pin.messageId)}
        <span class="pin-rail-segment" class:active={i === pins.length - 1 - index}></span>
      {/each}
    </span>
    <button
      class="pin-jump"
      type="button"
      onclick={() => {
        onJump(current.messageId);
        cursor += 1;
      }}
    >
      <strong>Pinned message{pins.length > 1 ? ` #${pins.length - index}` : ''}</strong>
      <span>{current.preview || 'Photo or file'}</span>
    </button>
    {#if canUnpin}
      <button
        aria-label={`Unpin ${current.preview || 'message'}`}
        title="Unpin"
        class="pin-remove"
        type="button"
        onclick={() => onUnpin(current.messageId)}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18">
          <path
            d="M6 6l12 12M18 6L6 18"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
          />
        </svg>
      </button>
    {/if}
  </div>
{/if}
