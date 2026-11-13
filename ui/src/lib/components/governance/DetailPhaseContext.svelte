<script lang="ts">
  import '#lib/components/styles/governance.css';

  /** web's DetailPhaseContext: kicker, phase title and the summary's first sentence.
   * ADR 0099: one line under the progress line, "Phase 2 of 5 · Production Plan". */
  interface Props {
    /** "Phase 2 of 5". */
    position?: string;
    kicker?: string;
    title: string;
    summary?: string;
  }

  let { position = '', kicker = '', title, summary = '' }: Props = $props();

  const sharpSummary = $derived(summary.match(/^.*?[.!?](?:\s|$)/)?.[0]?.trim() || summary.trim());
</script>

<header class="phase-context overview-phase-context">
  <div class="phase-line">
    {#if position}<span class="position">{position}</span>{/if}
    <h2>{title}</h2>
    {#if kicker}<p class="kicker">{kicker}</p>{/if}
  </div>
  {#if sharpSummary}
    <p class="summary">{sharpSummary}</p>
  {/if}
</header>
