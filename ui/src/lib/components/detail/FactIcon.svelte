<script lang="ts" module>
  export type FactIconName = 'when' | 'where' | 'people' | 'phase' | 'lineage';
</script>

<script lang="ts">
  /** The 20px line icons of a detail page's facts card. */
  let { name }: { name: FactIconName } = $props();
</script>

<svg
  aria-hidden="true"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
>
  {#if name === 'when'}
    <rect x="4" y="5" width="16" height="15" rx="2" /><path d="M4 9h16M8 3v4M16 3v4" />
  {:else if name === 'where'}
    <path d="M12 21s6.5-5.1 6.5-10a6.5 6.5 0 1 0-13 0c0 4.9 6.5 10 6.5 10Z" />
    <circle cx="12" cy="11" r="2.2" />
  {:else if name === 'people'}
    <circle cx="9" cy="8.5" r="3.2" /><path d="M3.5 19c.6-3.2 2.8-5 5.5-5s4.9 1.8 5.5 5" />
    <path d="M15.5 5.6a3 3 0 0 1 0 5.8M17.5 14.4c1.7.6 2.7 2.2 3 4.6" />
  {:else if name === 'phase'}
    <path d="M5 12h3l2.5-6 3 12 2.5-6h3" />
  {:else}
    <path d="M7 7h10v10" /><path d="M7 17 17 7" />
  {/if}
</svg>
