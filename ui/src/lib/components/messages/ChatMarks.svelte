<script lang="ts">
  /** web's pin and mute marks beside a chat's name (list rows and the chat header). */
  interface Props {
    pinned?: boolean | undefined;
    muted?: boolean | undefined;
    /** The header's mute mark (web's `header-mute`). */
    header?: boolean;
  }

  let { pinned, muted, header = false }: Props = $props();
</script>

{#if pinned}
  <span class="pin-mark" aria-label="Pinned">
    <svg aria-hidden="true" viewBox="0 0 24 24">
      <path
        d="M16 3l5 5-1.5 1.5-2.2-.7-3.1 3.1.8 3.6L12 13.5 8.2 17.3 6.7 15.8l3.8-3.8-2.1-3.1 3.1-3.1L11 3.5 16 3z"
        fill="currentColor"
      />
    </svg>
  </span>
{/if}
{#if muted}
  <span class:header-mute={header} class="muted-mark" aria-label="Muted" title="Muted">
    <svg aria-hidden="true" viewBox="0 0 24 24">
      <path d="M4 9v6h4l5 4V5L8 9H4z" fill="currentColor" />
      <path
        d="m16 9 5 6M21 9l-5 6"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
      />
    </svg>
  </span>
{/if}
