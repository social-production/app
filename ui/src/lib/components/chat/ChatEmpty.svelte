<script lang="ts">
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';

  /** A chat with no messages: its empty line, or skeleton rows while it loads (no copy yet). */
  let { copy }: { copy: string } = $props();
</script>

{#if copy}
  <div class="empty-state">{copy}</div>
{:else}
  <SkeletonRows count={3} kind="row" />
{/if}
