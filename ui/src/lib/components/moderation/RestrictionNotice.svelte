<script lang="ts">
  import type { Snippet } from 'svelte';

  import '#lib/components/styles/report.css';

  /**
   * web's ModerationRestrictionNotice: on a hidden item's own page the title (and a thread's or
   * post's body) blurs behind the serious-harm notice until the reader reveals it (ADR 0071).
   */
  let { active = false, children }: { active?: boolean; children: Snippet } = $props();

  let revealed = $state(false);
  const blurContent = $derived(active && !revealed);
</script>

{#if active}
  <div class="restriction-notice">
    <span>Serious harm report — content blurred until you choose to reveal it.</span>
    <button class="reveal-btn" type="button" onclick={() => (revealed = !revealed)}>
      {revealed ? 'Hide again' : 'Reveal'}
    </button>
  </div>
{/if}

<div class:blurred={blurContent} class="restriction-body">
  {@render children()}
</div>
