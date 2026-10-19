<script lang="ts">
  import '#lib/components/styles/governance.css';

  /** web's DetailPhaseContext: kicker, phase title and the summary's first sentence. */
  interface Props {
    kicker?: string;
    title: string;
    summary?: string;
  }

  let { kicker = '', title, summary = '' }: Props = $props();

  const sharpSummary = $derived(summary.match(/^.*?[.!?](?:\s|$)/)?.[0]?.trim() || summary.trim());
</script>

<header class="phase-context overview-phase-context">
  {#if kicker}
    <p class="kicker">{kicker}</p>
  {/if}
  <h2>{title}</h2>
  {#if sharpSummary}
    <p class="summary">{sharpSummary}</p>
  {/if}
</header>
