<script lang="ts">
  import PhraseGrid from '#lib/components/PhraseGrid.svelte';
  import { NAME_MAX } from '#lib/domain/people.ts';
  import {
    confirmationPositions,
    createAccount,
    explain,
    newPhrase,
    type AccountStep,
  } from '#lib/services/account.ts';

  import ProgressSteps from './ProgressSteps.svelte';

  let { onCancel }: { onCancel: () => void } = $props();

  const words = newPhrase();
  const positions = confirmationPositions();

  let step = $state<'name' | 'phrase' | 'confirm' | 'working'>('name');
  let name = $state('');
  let answers = $state(['', '', '']);
  let progress = $state<AccountStep>('checking');
  let error = $state<string>();

  const confirmed = $derived(
    positions.every((p, i) => answers[i]?.trim().toLowerCase() === words[p]),
  );

  async function finish(): Promise<void> {
    step = 'working';
    error = undefined;
    try {
      await createAccount(name.trim(), words, (s) => (progress = s));
    } catch (e) {
      error = explain(e);
      step = 'confirm';
    }
  }
</script>

{#if step === 'name'}
  <h2>Create your account</h2>
  <label>
    Your name (shown to others)
    <input bind:value={name} maxlength={NAME_MAX} autocomplete="name" />
  </label>
  <div class="row">
    <button class="primary" disabled={!name.trim()} onclick={() => (step = 'phrase')}>Next</button>
    <button onclick={onCancel}>Back</button>
  </div>
{:else if step === 'phrase'}
  <h2>Your recovery phrase</h2>
  <p>
    These 24 words <strong>are</strong> your account. Write them down on paper and keep them safe. Anyone
    with them can act as you; without them, a lost device means a lost account.
  </p>
  <PhraseGrid {words} />
  <button class="primary" onclick={() => (step = 'confirm')}>I've written them down</button>
{:else if step === 'confirm'}
  <h2>Check your phrase</h2>
  {#each positions as position, i (position)}
    <label>
      Word #{position + 1}
      <input bind:value={answers[i]} autocapitalize="none" autocomplete="off" spellcheck="false" />
    </label>
  {/each}
  {#if error}<p class="error">{error}</p>{/if}
  <div class="row">
    <button class="primary" disabled={!confirmed} onclick={finish}>Create account</button>
    <button onclick={() => (step = 'phrase')}>Show the words again</button>
  </div>
{:else}
  <ProgressSteps current={progress} />
{/if}
