<script lang="ts">
  import { resolve } from '$app/paths';

  import '#lib/components/styles/ux/dock.css';

  /** The round floating back button of the detail dock (ADR 0099) and the comment bar (ADR 0100). */
  function back(): void {
    if (history.length > 1) history.back();
    else location.assign(resolve('/'));
  }
</script>

<button class="dock-back" type="button" aria-label="Back" title="Back" onclick={back}>
  <svg aria-hidden="true" viewBox="0 0 24 24"
    ><path
      d="M14.5 5.5 8 12l6.5 6.5"
      fill="none"
      stroke="currentColor"
      stroke-width="2.2"
      stroke-linecap="round"
      stroke-linejoin="round"
    /></svg
  >
</button>
