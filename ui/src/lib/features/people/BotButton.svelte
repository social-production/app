<script lang="ts">
  /** web's "Bot" action button (profile action row, ADR 0074). */
  interface Props {
    name: string;
    marked: boolean;
    canMark: boolean;
    busy: boolean;
    onToggle: () => void;
  }

  let { name, marked, canMark, busy, onToggle }: Props = $props();
</script>

<button
  aria-label={marked ? `Clear bot mark for ${name}` : `Mark ${name} as a bot`}
  aria-pressed={marked}
  class="action bot"
  class:on={marked}
  disabled={busy || (!marked && !canMark)}
  title={marked
    ? 'Clear bot mark'
    : canMark
      ? 'Mark as bot'
      : 'Marking as a bot opens once your own votes count fully.'}
  type="button"
  onclick={onToggle}
>
  <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18"
    ><rect
      x="5"
      y="8"
      width="14"
      height="11"
      rx="3"
      fill="none"
      stroke="currentColor"
      stroke-width="1.8"
    /><path
      d="M12 4v4M9.5 13h.01M14.5 13h.01M10 16.5h4"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
    /><circle cx="12" cy="3.5" r="1" fill="currentColor" /></svg
  >
  <span class="action-text">{marked ? 'Marked bot' : 'Bot'}</span>
</button>
