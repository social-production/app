<script lang="ts">
  import type { AccountStep } from '#lib/services/account.ts';

  let { current }: { current: AccountStep } = $props();

  const steps: { id: AccountStep; label: string }[] = [
    { id: 'checking', label: 'Proving you are not a bot (a few seconds)…' },
    { id: 'passkey', label: 'Save a passkey on this device when your browser asks…' },
    { id: 'connecting', label: 'Setting up your node…' },
  ];
  const index = $derived(steps.findIndex((s) => s.id === current));
</script>

<ol class="stack steps">
  {#each steps as step, i (step.id)}
    <li class:done={i < index} class:active={i === index}>{step.label}</li>
  {/each}
</ol>

<style>
  .steps {
    padding-left: 20px;
    color: var(--text-soft);
  }
  .active {
    color: var(--text-main);
    font-weight: 600;
  }
  .done {
    color: var(--brand-strong);
  }
</style>
