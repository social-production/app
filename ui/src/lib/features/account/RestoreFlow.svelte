<script lang="ts">
  import {
    explain,
    phraseIsValid,
    restore,
    splitWords,
    type AccountStep,
  } from '#lib/services/account.ts';

  import ProgressSteps from './ProgressSteps.svelte';

  let { onCancel }: { onCancel: () => void } = $props();

  let text = $state('');
  let working = $state(false);
  let progress = $state<AccountStep>('checking');
  let error = $state<string>();

  const words = $derived(splitWords(text));
  const valid = $derived(phraseIsValid(words));

  async function go(): Promise<void> {
    working = true;
    error = undefined;
    try {
      await restore(words, (s) => (progress = s));
    } catch (e) {
      error = explain(e);
      working = false;
    }
  }
</script>

{#if working}
  <ProgressSteps current={progress} />
{:else}
  <h2>Restore your account</h2>
  <p class="muted">
    Enter your 24 words. This device gets a new passkey; your identity and history stay the same.
  </p>
  <label>
    Recovery phrase
    <textarea bind:value={text} rows="5" autocapitalize="none" spellcheck="false"></textarea>
  </label>
  <p class="muted">
    {words.length}/24 words{words.length === 24 && !valid ? ' — check the spelling' : ''}
  </p>
  {#if error}<p class="error">{error}</p>{/if}
  <div class="row">
    <button class="primary" disabled={!valid} onclick={go}>Restore</button>
    <button onclick={onCancel}>Back</button>
  </div>
{/if}
