<script lang="ts">
  import {
    explain,
    phraseIsValid,
    signInWithPhrase,
    splitWords,
    type AccountStep,
  } from '#lib/services/account.ts';

  import ProgressSteps from './ProgressSteps.svelte';

  let { onCancel }: { onCancel: () => void } = $props();

  let text = $state('');
  let working = $state(false);
  let progress = $state<AccountStep>('connecting');
  let error = $state<string>();

  const words = $derived(splitWords(text));
  const valid = $derived(phraseIsValid(words));

  async function go(): Promise<void> {
    working = true;
    error = undefined;
    try {
      await signInWithPhrase(words, (s) => (progress = s));
    } catch (e) {
      error = explain(e);
      working = false;
    }
  }
</script>

{#if working}
  <ProgressSteps current={progress} />
{:else}
  <h2>Sign in with your recovery phrase</h2>
  <p class="muted">
    Type or paste your 24 words. This device stays signed in until you sign out. If your account
    lives on another site or app, it is brought here — same identity.
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
    <button class="primary" disabled={!valid} onclick={go}>Sign in</button>
    <button onclick={onCancel}>Back</button>
  </div>
{/if}
