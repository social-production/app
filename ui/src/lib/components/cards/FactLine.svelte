<script lang="ts">
  import type { Snippet } from 'svelte';

  /** A when or where line under a detail page's title: a small icon, then the fact (ADR 0099). */
  let { icon, children }: { icon: 'when' | 'where'; children: Snippet } = $props();
</script>

<p class="live-fact fact-line">
  <svg aria-hidden="true" viewBox="0 0 24 24">
    {#if icon === 'when'}
      <rect
        x="4"
        y="5"
        width="16"
        height="15"
        rx="2"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
      />
      <path
        d="M4 9h16M8 3v4M16 3v4"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
      />
    {:else}
      <path
        d="M12 21s6.5-5.1 6.5-10a6.5 6.5 0 1 0-13 0c0 4.900 6.5 10 6.5 10Z"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linejoin="round"
      />
      <circle cx="12" cy="11" r="2.2" fill="none" stroke="currentColor" stroke-width="1.8" />
    {/if}
  </svg>
  <span>{@render children()}</span>
</p>
