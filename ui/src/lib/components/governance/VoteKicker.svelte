<script lang="ts">
  /** The vote sheet's top line (owner, 2026-11-18): the kind in green, then who and when. */
  let {
    label,
    meta = '',
    status = null,
  }: { label: string; meta?: string; status?: string | null } = $props();
</script>

<p class="vote-kicker">
  <b>{label}</b>{#if status}<span class="vote-kicker-status">· {status}</span>{/if}{#if meta}<span
      >· {meta}</span
    >{/if}
</p>
