<script lang="ts">
  import { pausedAgents } from './pausedAgents.ts';

  /** web's PausedBadge: a small "Paused" tag after a name that a bot review found to be a bot. */
  let { agent }: { agent: string | null | undefined } = $props();

  const paused = pausedAgents();
  const explanation = "Paused: a community bot review found this account is a bot. It's read-only.";

  import '#lib/components/styles/paused.css';
</script>

{#if agent && paused.has(agent)}
  <span class="paused-badge" title={explanation} aria-label={explanation}>
    <span class="dot" aria-hidden="true"></span>
    <span class="text">Paused</span>
  </span>
{/if}
