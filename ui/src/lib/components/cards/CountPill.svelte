<script lang="ts">
  import '#lib/components/styles/discussion.css';

  /** web's CountPill: a speech-bubble pill ("3 comments"). */
  let { label }: { label: string } = $props();

  const count = $derived(/^(\d+)/.exec(label)?.[1] ?? '');
  const compact = $derived(count ? compactCount(Number(count)) : '');

  /** web's formatCompactCount: 999, 1.2k, 12k, 1m, 1.5m. */
  function compactCount(value: number): string {
    const n = Math.max(0, Math.round(value));
    const short = (x: number): string => (x % 1 === 0 ? x.toFixed(0) : x.toFixed(1));
    if (n < 1_000) return String(n);
    if (n < 10_000) return `${short((Math.round(n / 100) * 100) / 1_000)}k`;
    if (n < 1_000_000) {
      const thousands = Math.round(n / 1_000);
      return thousands >= 1_000 ? '1m' : `${thousands}k`;
    }
    return `${short((Math.round(n / 100_000) * 100_000) / 1_000_000)}m`;
  }
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
  {#if compact}<span aria-hidden="true" class="pill-count">{compact}</span>{/if}
</span>
