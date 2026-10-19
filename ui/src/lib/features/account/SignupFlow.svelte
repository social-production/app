<script lang="ts">
  import PhraseGrid from '#lib/components/PhraseGrid.svelte';
  import { HANDLE_MAX, handleProblem } from '#lib/domain/handle.ts';
  import {
    confirmationPositions,
    createAccount,
    newPhrase,
    type AccountStep,
  } from '#lib/services/account.ts';
  import { explain } from '#lib/services/errors.ts';

  import ProgressSteps from './ProgressSteps.svelte';

  const words = newPhrase();
  const positions = confirmationPositions();

  let step = $state<'name' | 'phrase' | 'confirm' | 'working'>('name');
  let name = $state('');
  const problem = $derived(handleProblem(name));
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
  <form class="stack" onsubmit={(e) => (e.preventDefault(), (step = 'phrase'))}>
    <label>
      <span class="field-label">Username</span>
      <input
        bind:value={name}
        maxlength={HANDLE_MAX}
        autocomplete="username"
        autocapitalize="none"
      />
    </label>
    <!-- One reserved line (as on web), so typing doesn't push the button down. -->
    <div class="handle-line">
      {#if problem && name.trim().length >= 3}<p class="status-note">{problem}</p>{/if}
    </div>
    <div class="button-row">
      <button class="button-primary" disabled={!!problem} type="submit">Next</button>
    </div>
  </form>
{:else if step === 'phrase'}
  <div class="stack">
    <p class="field-label">Your recovery phrase</p>
    <p>
      These 24 words <strong>are</strong> your account — you log in with them. Write them down on paper
      and keep them safe. Anyone with them can act as you; without them, a lost device means a lost account.
    </p>
    <PhraseGrid {words} />
    <div class="button-row">
      <button class="button-primary" type="button" onclick={() => (step = 'confirm')}
        >I've written them down</button
      >
    </div>
  </div>
{:else if step === 'confirm'}
  <form class="stack" onsubmit={(e) => (e.preventDefault(), void finish())}>
    <p class="field-label">Check your phrase</p>
    {#each positions as position, i (position)}
      <label>
        <span class="field-label">Word #{position + 1}</span>
        <input
          bind:value={answers[i]}
          autocapitalize="none"
          autocomplete="off"
          spellcheck="false"
        />
      </label>
    {/each}
    {#if error}<p class="status-note">{error}</p>{/if}
    <div class="button-row">
      <button class="button-primary" disabled={!confirmed} type="submit">Create account</button>
      <button class="button-secondary" type="button" onclick={() => (step = 'phrase')}
        >Show the words again</button
      >
    </div>
  </form>
{:else}
  <ProgressSteps current={progress} />
{/if}
