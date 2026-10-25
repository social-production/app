<script lang="ts">
  import { formatCompactCount } from '#lib/domain/count.ts';

  /** web's CountPill: an icon and a compact count; `label` ("3 comments") is read aloud. */
  let { label }: { label: string } = $props();

  const compactCount = $derived.by(() => {
    const match = /^(\d+)/.exec(label);
    return match ? formatCompactCount(Number(match[1])) : '';
  });
</script>

<span class="pill" aria-label={label}>
  <span aria-hidden="true" class="pill-icon">
    <svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M2.5 3.25A1.25 1.25 0 0 1 3.75 2h8.5a1.25 1.25 0 0 1 1.25 1.25v5.5A1.25 1.25 0 0 1 12.25 10H9.2l-2.1 2.45a.5.5 0 0 1-.8-.4V10H3.75A1.25 1.25 0 0 1 2.5 8.75v-5.5Z"
        stroke="currentColor"
        stroke-width="1.2"
        stroke-linejoin="round"
      />
    </svg>
  </span>
  <span class="pill-text">{label}</span>
  {#if compactCount}
    <span aria-hidden="true" class="pill-count">{compactCount}</span>
  {/if}
</span>
